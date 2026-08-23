use super::{Provider, ProviderResult, ResultKind};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use encoding_rs::UTF_16LE;
use flate2::read::ZlibDecoder;
use ripemd::{Digest, Ripemd128};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

#[allow(dead_code)]
const MDX_MAGIC: &[u8] = b"MDict dictionary file\x00";
const RECORD_BLOCK_COMP_NONE: u32 = 0x00000000;
const RECORD_BLOCK_COMP_LZO: u32 = 0x01000000;
const RECORD_BLOCK_COMP_ZLIB: u32 = 0x02000000;

#[derive(Debug, Clone)]
struct KeyEntry {
    headword_lower: String,
    headword: String,
    record_start: u64,
    record_end: u64,
}

pub struct MdxDict {
    path: PathBuf,
    /// Sorted by headword_lower for binary search
    index: Vec<KeyEntry>,
    /// (block_offset, block_compressed_size, block_decompressed_size)
    record_blocks: Vec<(u64, u64, u64)>,
    record_section_offset: u64,
}

impl MdxDict {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)
            .with_context(|| format!("Cannot open MDX file: {}", path.display()))?;
        let mut reader = BufReader::new(file);

        let (index, record_blocks, record_section_offset) = parse_mdx(&mut reader, &path)
            .with_context(|| format!("Failed to parse MDX file: {}", path.display()))?;

        Ok(MdxDict {
            path,
            index,
            record_blocks,
            record_section_offset,
        })
    }

    pub fn lookup_exact(&self, query: &str) -> Result<Option<String>> {
        let q = query.to_lowercase();

        let pos = self
            .index
            .partition_point(|e| e.headword_lower.as_str() < q.as_str());

        let entry = match self.index.get(pos).filter(|e| e.headword_lower == q) {
            Some(entry) => entry,
            None => return Ok(None),
        };

        let html = self.read_record(entry)?;
        Ok(Some(html))
    }

    fn read_record(&self, entry: &KeyEntry) -> Result<String> {
        let mut file = File::open(&self.path)?;

        let mut block_data_offset: u64 = 0;
        for (blk_file_off, blk_comp_sz, blk_decomp_sz) in &self.record_blocks {
            let blk_end = block_data_offset + blk_decomp_sz;

            if entry.record_start >= block_data_offset && entry.record_start < blk_end {
                file.seek(SeekFrom::Start(self.record_section_offset + blk_file_off))?;

                let comp_len = checked_len("record block comp_size", *blk_comp_sz)?;
                let mut comp_buf = vec![0u8; comp_len];
                file.read_exact(&mut comp_buf)?;

                let decompressed = decompress_block(&comp_buf, *blk_decomp_sz)?;

                let local_start = (entry.record_start - block_data_offset) as usize;
                let local_end = ((entry.record_end - block_data_offset) as usize)
                    .min(decompressed.len());

                let record_bytes = &decompressed[local_start..local_end];
                let html = decode_content(record_bytes);
                return Ok(html);
            }

            block_data_offset += blk_decomp_sz;
        }

        bail!("Record not found in any block for entry: {}", entry.headword)
    }
}

/// Maximum size we're willing to allocate for a single length-prefixed
/// section while parsing untrusted file input. MDX headers, key index
/// blocks, and individual record blocks legitimately reach a few hundred
/// MB for large dictionaries, but never gigabytes — this bound exists
/// solely to fail fast with a clear error instead of attempting a
/// multi-gigabyte allocation when a length field is corrupt or was
/// misread (see the header_len byte-order bug this replaces).
const MAX_SECTION_BYTES: u64 = 512 * 1024 * 1024;

fn checked_len(label: &str, len: u64) -> Result<usize> {
    if len > MAX_SECTION_BYTES {
        bail!(
            "MDX {} field is implausibly large ({} bytes) — file is corrupt or its format \
             differs from what this parser expects",
            label,
            len
        );
    }
    Ok(len as usize)
}

fn parse_mdx(
    reader: &mut BufReader<File>,
    _path: &Path,
) -> Result<(Vec<KeyEntry>, Vec<(u64, u64, u64)>, u64)> {
    // NOTE: header_len is BIG-endian per the MDict container spec, not
    // little-endian. Misreading this as LE was the root cause of every
    // MDX file in this codebase failing to load: it turns a legitimate
    // value like 2808 into billions, which then blows up the very next
    // allocation (`vec![0u8; header_len as usize]`) before any dictionary
    // content is even touched.
    let header_len = read_u32_be(reader)?;
    let header_len = checked_len("header_len", header_len as u64)?;
    let mut header_bytes = vec![0u8; header_len];
    reader.read_exact(&mut header_bytes)?;
    let mut _adler = [0u8; 4];
    reader.read_exact(&mut _adler)?;

    let header_str = String::from_utf16_le_lossy(&header_bytes);
    let version: f32 = parse_header_attr(&header_str, "GeneratedByEngineVersion")
        .unwrap_or_else(|| "2.0".to_string())
        .parse()
        .unwrap_or(2.0);

    // This parser implements the MDX v2.0 container layout (8-byte block
    // counts/sizes throughout). v1.2 uses a different, incompatible layout
    // (4-byte counts, 1-byte key-length prefixes, GBK-only encoding) that
    // is not implemented here. Rather than silently misapply the v2.0
    // field widths to a v1.2 file — which reliably produces garbage
    // offsets and either a panic or a confusing "corrupt file" error four
    // steps removed from the actual cause — fail immediately with a clear
    // message naming the unsupported version.
    if version < 2.0 {
        bail!(
            "MDX file reports GeneratedByEngineVersion=\"{}\", but only v2.0+ is supported",
            version
        );
    }

    let encrypted: u8 = parse_header_attr(&header_str, "Encrypted")
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    // MDX v2 uses 8-byte counts
    let num_blocks = read_u64_be(reader)?;
    let _num_entries = read_u64_be(reader)?;
    let key_index_decomp_size = read_u64_be(reader)?;
    let key_index_comp_size = read_u64_be(reader)?;
    let key_blocks_total_size = read_u64_be(reader)?;
    let mut _ck = [0u8; 4];
    reader.read_exact(&mut _ck)?;

    let key_index_comp_len = checked_len("key_index_comp_size", key_index_comp_size)?;
    let mut key_index_comp = vec![0u8; key_index_comp_len];
    reader.read_exact(&mut key_index_comp)?;

    // When Encrypted has bit 0x02 set, the key-block-info section (the
    // block that parse_key_block_info reads below — num_entries per block,
    // first/last headword, comp/decomp sizes) is RC4-encrypted with a key
    // derived from the block's own Adler-32 checksum. Everything else
    // (the key blocks themselves, record blocks) is untouched. Without
    // this decryption step, comp_sz/decomp_sz below are read out of
    // ciphertext and come out as effectively random u64s, which is what
    // was actually causing the "corrupt / EOF" failures on top of the
    // header_len bug — LDOCE-family and vocabulary.com dictionaries both
    // commonly set Encrypted="2".
    let key_index_decomp = if encrypted & 0x02 != 0 {
        let decrypted = decrypt_key_block_info(&key_index_comp)?;
        decompress_block(&decrypted, key_index_decomp_size)?
    } else {
        decompress_block(&key_index_comp, key_index_decomp_size)?
    };

    let key_block_sizes = parse_key_block_info(&key_index_decomp, num_blocks)?;

    let mut index: Vec<KeyEntry> = Vec::new();
    {
        let key_blocks_start = reader.stream_position()?;
        for (comp_sz, decomp_sz) in &key_block_sizes {
            let comp_len = checked_len("key block comp_size", *comp_sz)?;
            let mut comp = vec![0u8; comp_len];
            reader.read_exact(&mut comp)?;
            let decomp = decompress_block(&comp, *decomp_sz)?;
            parse_key_block(&decomp, &mut index)?;
        }
        let consumed = reader.stream_position()? - key_blocks_start;
        if consumed != key_blocks_total_size {
            log::warn!(
                "MDX key blocks: expected {} bytes, consumed {}",
                key_blocks_total_size, consumed
            );
        }
    }

    index.sort_by(|a, b| a.headword_lower.cmp(&b.headword_lower));

    let num_record_blocks = read_u64_be(reader)?;
    let _num_entries2 = read_u64_be(reader)?;
    let _record_index_size = read_u64_be(reader)?;
    let _record_blocks_total_size = read_u64_be(reader)?;

    let mut record_blocks: Vec<(u64, u64, u64)> = Vec::new();
    {
        let mut running_offset: u64 = 0;
        for _ in 0..num_record_blocks {
            let comp_sz = read_u64_be(reader)?;
            let decomp_sz = read_u64_be(reader)?;
            record_blocks.push((running_offset, comp_sz, decomp_sz));
            running_offset += comp_sz;
        }
    }

    let record_section_offset = reader.stream_position()?;

    fill_record_ends(&mut index);

    Ok((index, record_blocks, record_section_offset))
}

/// Decrypts an MDX key-block-info section (Encrypted has bit 0x02 set, i.e.
/// `Encrypted & 2 != 0`).
///
/// Per the MDict format spec (zhansliu/writemdict fileformat.md, "Keyword
/// index encryption"): the key-block-info's `comp_type` and `checksum`
/// header (bytes 0..8 of the compressed section) are left untouched; only
/// the `compressed_data` that follows (byte 8 onward) is encrypted, via:
///
///   key = ripemd128(checksum_bytes ++ "\x95\x36\x00\x00")
///   for i, byte in enumerate(compressed_data):
///       byte = SWAPNIBBLE(byte ^ i ^ key[i % keylen] ^ previous)
///       previous = byte  (the *encrypted* output byte, not the input)
///
/// This is NOT RC4 — an earlier version of this function used RC4 keyed
/// with MD5, which is wrong on two independent counts (wrong cipher, wrong
/// hash) and reliably produces garbage bytes that zlib then rejects with
/// "corrupt deflate stream". SWAPNIBBLE swaps the high/low nibbles of a
/// byte: ((b >> 4) | (b << 4)) & 0xFF.
///
/// Decryption is this cipher's own exact inverse, run in reverse order per
/// byte (since `previous` depends on the *ciphertext* byte, which is now
/// the input rather than the output):
///
///   plain_byte = SWAPNIBBLE(cipher_byte) ^ i ^ key[i % keylen] ^ previous
///   previous = cipher_byte
fn decrypt_key_block_info(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 8 {
        bail!(
            "Encrypted key block info is too short to contain a header ({} bytes)",
            data.len()
        );
    }

    // checksum_bytes is the 4-byte ADLER32 checksum stored at data[4..8]
    // (bytes 0..4 are comp_type, bytes 4..8 are the checksum — see the
    // "Compression" section of the format spec).
    let mut key_material = [0u8; 8];
    key_material[0..4].copy_from_slice(&data[4..8]);
    key_material[4..8].copy_from_slice(&[0x95, 0x36, 0x00, 0x00]);
    let key = Ripemd128::digest(key_material);

    let mut out = data.to_vec();
    mdict_keyword_index_decrypt_in_place(&key, &mut out[8..]);
    Ok(out)
}

/// Inverse of the MDict keyword-index-encryption byte cipher described
/// above. `key` is typically 16 bytes (RIPEMD-128 output) but any nonzero
/// length is handled via `i % key.len()`, matching the reference `keylen`
/// modulus in the spec's C implementation.
fn mdict_keyword_index_decrypt_in_place(key: &[u8], data: &mut [u8]) {
    if key.is_empty() {
        return;
    }
    let mut previous: u8 = 0x36;
    for (i, byte) in data.iter_mut().enumerate() {
        let cipher_byte = *byte;
        let swapped = (cipher_byte >> 4) | (cipher_byte << 4);
        let plain = swapped ^ (i as u8) ^ key[i % key.len()] ^ previous;
        previous = cipher_byte;
        *byte = plain;
    }
}

fn parse_key_block_info(data: &[u8], num_blocks: u64) -> Result<Vec<(u64, u64)>> {
    let num_blocks = checked_len("num_blocks", num_blocks)?;
    let mut sizes: Vec<(u64, u64)> = Vec::with_capacity(num_blocks);
    let mut pos = 0usize;

    for _ in 0..num_blocks {
        // num_entries(8) + first_size(2) + first_word + last_size(2) + last_word + comp_size(8) + decomp_size(8)
        if pos + 8 > data.len() {
            bail!(
                "MDX key block info truncated: expected {} blocks, ran out of data after {}",
                num_blocks,
                sizes.len()
            );
        }
        pos += 8;

        if pos + 2 > data.len() {
            bail!("MDX key block info truncated while reading first_size");
        }
        let first_sz = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        let first_span = (first_sz + 1) * 2;
        if pos + first_span > data.len() {
            bail!(
                "MDX key block info: first_word span ({} bytes at offset {}) exceeds section \
                 length ({}) — the section may not actually be encrypted the way this parser \
                 assumed, or the file is corrupt",
                first_span, pos, data.len()
            );
        }
        pos += first_span;

        if pos + 2 > data.len() {
            bail!("MDX key block info truncated while reading last_size");
        }
        let last_sz = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        let last_span = (last_sz + 1) * 2;
        if pos + last_span > data.len() {
            bail!(
                "MDX key block info: last_word span ({} bytes at offset {}) exceeds section \
                 length ({})",
                last_span, pos, data.len()
            );
        }
        pos += last_span;

        if pos + 16 > data.len() {
            bail!("MDX key block info truncated while reading comp_size/decomp_size");
        }
        let comp_sz = u64::from_be_bytes(data[pos..pos + 8].try_into()?);
        pos += 8;
        let decomp_sz = u64::from_be_bytes(data[pos..pos + 8].try_into()?);
        pos += 8;
        sizes.push((comp_sz, decomp_sz));
    }

    Ok(sizes)
}

fn parse_key_block(data: &[u8], index: &mut Vec<KeyEntry>) -> Result<()> {
    let mut pos = 0usize;

    while pos < data.len() {
        if pos + 8 > data.len() { break; }
        let record_offset = u64::from_be_bytes(data[pos..pos+8].try_into()?);
        pos += 8;

        let mut chars: Vec<u16> = Vec::new();
        while pos + 2 <= data.len() {
            let ch = u16::from_le_bytes([data[pos], data[pos+1]]);
            pos += 2;
            if ch == 0 { break; }
            chars.push(ch);
        }

        let headword = String::from_utf16_lossy(&chars).to_string();

        index.push(KeyEntry {
            headword_lower: headword.to_lowercase(),
            headword,
            record_start: record_offset,
            record_end: 0, // patched by fill_record_ends
        });
    }

    Ok(())
}

/// Patches each entry's record_end by chaining adjacent offsets, sorted by
/// record_start. The last entry gets u64::MAX; read_record clamps it.
fn fill_record_ends(index: &mut Vec<KeyEntry>) {
    let len = index.len();
    if len == 0 { return; }

    let mut by_offset: Vec<usize> = (0..len).collect();
    by_offset.sort_by_key(|&i| index[i].record_start);

    for w in by_offset.windows(2) {
        let (cur, next) = (w[0], w[1]);
        index[cur].record_end = index[next].record_start;
    }
    if let Some(&last) = by_offset.last() {
        index[last].record_end = u64::MAX;
    }
}

fn decompress_block(data: &[u8], decompressed_size: u64) -> Result<Vec<u8>> {
    if data.len() < 8 {
        anyhow::bail!("MDX block too short to contain a compression header ({} bytes)", data.len());
    }
    let comp_type = u32::from_be_bytes(data[0..4].try_into()?);
    let payload = &data[8..]; // bytes 4..8 are an adler32 checksum, skipped
    let decompressed_len = checked_len("decompressed_size", decompressed_size)?;

    match comp_type {
        RECORD_BLOCK_COMP_NONE => Ok(payload.to_vec()),
        RECORD_BLOCK_COMP_LZO => {
            // LZO1X is not self-terminating like zlib — decompressed_size
            // must be known ahead of time.
            let mut out = vec![0u8; decompressed_len];
            lzo1x::decompress(payload, &mut out)
                .map_err(|e| anyhow::anyhow!("LZO decompression failed: {:?}", e))?;
            Ok(out)
        }
        RECORD_BLOCK_COMP_ZLIB => {
            let mut decoder = ZlibDecoder::new(payload);
            let mut out = Vec::with_capacity(decompressed_len);
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        other => {
            bail!("Unsupported MDX compression type: 0x{:08X}", other)
        }
    }
}

fn decode_content(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes[1] == 0 {
        let (decoded, _, _) = UTF_16LE.decode(bytes);
        decoded.trim_end_matches('\0').to_string()
    } else {
        String::from_utf8_lossy(bytes)
            .trim_end_matches('\0')
            .to_string()
    }
}

fn read_u32_be(r: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_u64_be(r: &mut impl Read) -> Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

fn parse_header_attr(header: &str, key: &str) -> Option<String> {
    let needle = format!("{}=\"", key);
    let start = header.find(&needle)? + needle.len();
    let end = header[start..].find('"')? + start;
    Some(header[start..end].to_string())
}

trait Utf16LeExt {
    fn from_utf16_le_lossy(bytes: &[u8]) -> String;
}
impl Utf16LeExt for String {
    fn from_utf16_le_lossy(bytes: &[u8]) -> String {
        let words: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&words).to_string()
    }
}

/// MDD files (audio/image/stylesheet resources bundled with an MDX
/// dictionary) share the MDX container format, so `parse_mdx` is reused;
/// only the semantics differ — keys are resource paths, records raw bytes.
pub struct MddDict {
    path: PathBuf,
    /// KeyEntry reused here; `headword` holds the resource path.
    index: Vec<KeyEntry>,
    record_blocks: Vec<(u64, u64, u64)>,
    record_section_offset: u64,
}

impl MddDict {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)
            .with_context(|| format!("Cannot open MDD file: {}", path.display()))?;
        let mut reader = BufReader::new(file);

        let (index, record_blocks, record_section_offset) = parse_mdx(&mut reader, &path)?;

        Ok(MddDict {
            path,
            index,
            record_blocks,
            record_section_offset,
        })
    }

    /// MDD entries are conventionally stored with a leading backslash and
    /// backslash separators regardless of build platform, but this varies
    /// between dictionaries — try a few common variants.
    pub fn lookup_resource(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let normalized_fwd = name.trim_start_matches(['/', '\\']);
        let normalized_back = normalized_fwd.replace('/', "\\");

        let candidates = [
            name.to_string(),
            format!("\\{}", normalized_back),
            format!("/{}", normalized_fwd),
            normalized_back.clone(),
            normalized_fwd.to_string(),
        ];

        let mut tried = std::collections::HashSet::new();
        for cand in &candidates {
            let cand_lower = cand.to_lowercase();
            if !tried.insert(cand_lower.clone()) {
                continue;
            }
            let pos = self
                .index
                .partition_point(|e| e.headword_lower.as_str() < cand_lower.as_str());
            if let Some(entry) = self.index.get(pos).filter(|e| e.headword_lower == cand_lower) {
                return Ok(Some(self.read_record_bytes(entry)?));
            }
        }
        Ok(None)
    }

    fn read_record_bytes(&self, entry: &KeyEntry) -> Result<Vec<u8>> {
        let mut file = File::open(&self.path)?;

        let mut block_data_offset: u64 = 0;
        for (blk_file_off, blk_comp_sz, blk_decomp_sz) in &self.record_blocks {
            let blk_end = block_data_offset + blk_decomp_sz;

            if entry.record_start >= block_data_offset && entry.record_start < blk_end {
                file.seek(SeekFrom::Start(self.record_section_offset + blk_file_off))?;

                let comp_len = checked_len("MDD record block comp_size", *blk_comp_sz)?;
                let mut comp_buf = vec![0u8; comp_len];
                file.read_exact(&mut comp_buf)?;

                let decompressed = decompress_block(&comp_buf, *blk_decomp_sz)?;

                let local_start = (entry.record_start - block_data_offset) as usize;
                let local_end = ((entry.record_end - block_data_offset) as usize)
                    .min(decompressed.len());

                return Ok(decompressed[local_start..local_end].to_vec());
            }

            block_data_offset += blk_decomp_sz;
        }

        bail!("MDD resource not found in any block: {}", entry.headword)
    }
}

pub struct MdxProvider {
    id: String,
    label: String,
    dict: Arc<Mutex<MdxDict>>,
}

impl MdxProvider {
    pub fn new(path: impl AsRef<Path>, label: String, id: String) -> Result<Self> {
        let dict = MdxDict::open(path)?;
        Ok(MdxProvider {
            id,
            label,
            dict: Arc::new(Mutex::new(dict)),
        })
    }
}

#[async_trait]
impl Provider for MdxProvider {
    fn id(&self) -> &str { &self.id }
    fn label(&self) -> &str { &self.label }

    async fn lookup(&self, query: &str) -> anyhow::Result<Option<ProviderResult>> {
        let dict = self.dict.lock().await;
        match dict.lookup_exact(query)? {
            None => Ok(None),
            Some(html) => Ok(Some(ProviderResult {
                provider_id: self.id.clone(),
                provider_label: self.label.clone(),
                kind: ResultKind::Dict,
                content: sanitize_mdx_html(html),
                phonetic: None,
                source_lang: None,
            })),
        }
    }
}

/// Strip potentially dangerous tags/attrs from MDX HTML output.
fn sanitize_mdx_html(html: String) -> String {
    let re_script = regex_lite::Regex::new(r"(?si)<script[^>]*>.*?</script>").unwrap();
    let html = re_script.replace_all(&html, "").to_string();
    let re_on = regex_lite::Regex::new(r#"(?i)\s+on\w+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)"#).unwrap();
    let html = re_on.replace_all(&html, "").to_string();
    html.replace("sound://", "wispet://mdd/sound/")
        .replace("entry://", "#")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Builds a minimal, well-formed MDX v2.0 file with a single entry,
    /// exercising the same encrypted-key-block-info path real dictionaries
    /// like LDOCE6 and vocabulary.com's MDX use (Encrypted="2"). Adler-32
    /// checksums are zeroed since nothing in this parser validates them.
    fn build_synthetic_mdx(encrypted: bool, headword: &str, html: &str) -> Vec<u8> {
        let encrypted_attr = if encrypted { "2" } else { "0" };
        let header_xml = format!(
            "<Dictionary GeneratedByEngineVersion=\"2.0\" RequiredEngineVersion=\"2.0\" \
             Encrypted=\"{encrypted_attr}\" Format=\"Html\" Encoding=\"UTF-8\"/>\r\n"
        );
        let header_utf16: Vec<u8> = header_xml
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .chain(std::iter::once(0u16).flat_map(|u| u.to_le_bytes())) // trailing NUL
            .collect();

        let mut out = Vec::new();
        // header_len (BE) + header + adler32(4, unchecked)
        out.extend_from_slice(&(header_utf16.len() as u32).to_be_bytes());
        out.extend_from_slice(&header_utf16);
        out.extend_from_slice(&[0u8; 4]);

        // --- build one key block containing one entry ---
        let mut key_block_plain = Vec::new();
        key_block_plain.extend_from_slice(&0u64.to_be_bytes()); // record_offset = 0
        key_block_plain.extend(headword.encode_utf16().flat_map(|u| u.to_le_bytes()));
        key_block_plain.extend_from_slice(&0u16.to_le_bytes()); // NUL terminator

        let key_block_comp = zlib_compress(&key_block_plain);
        let mut key_block_wrapped = Vec::new();
        key_block_wrapped.extend_from_slice(&RECORD_BLOCK_COMP_ZLIB.to_be_bytes());
        key_block_wrapped.extend_from_slice(&[0u8; 4]); // adler32, unchecked
        key_block_wrapped.extend_from_slice(&key_block_comp);

        // --- build key-block-info describing that one block ---
        let first_word_u16: Vec<u16> = headword.encode_utf16().collect();
        let mut kbi_plain = Vec::new();
        kbi_plain.extend_from_slice(&1u64.to_be_bytes()); // num_entries in this block
        kbi_plain.extend_from_slice(&(first_word_u16.len() as u16).to_be_bytes());
        for u in &first_word_u16 {
            kbi_plain.extend_from_slice(&u.to_le_bytes());
        }
        kbi_plain.extend_from_slice(&0u16.to_le_bytes()); // NUL after first_word
        kbi_plain.extend_from_slice(&(first_word_u16.len() as u16).to_be_bytes());
        for u in &first_word_u16 {
            kbi_plain.extend_from_slice(&u.to_le_bytes());
        }
        kbi_plain.extend_from_slice(&0u16.to_le_bytes()); // NUL after last_word
        kbi_plain.extend_from_slice(&(key_block_wrapped.len() as u64).to_be_bytes()); // comp_size
        kbi_plain.extend_from_slice(&(key_block_plain.len() as u64).to_be_bytes()); // decomp_size

        let kbi_comp_payload = zlib_compress(&kbi_plain);
        let mut kbi_wrapped = Vec::new();
        kbi_wrapped.extend_from_slice(&RECORD_BLOCK_COMP_ZLIB.to_be_bytes());
        kbi_wrapped.extend_from_slice(&[0u8; 4]); // adler32, unchecked
        kbi_wrapped.extend_from_slice(&kbi_comp_payload);

        let kbi_final = if encrypted {
            encrypt_key_block_info_for_test(&kbi_wrapped)
        } else {
            kbi_wrapped
        };

        // key section header: num_blocks, num_entries, key_index_decomp_size,
        // key_index_comp_size, key_blocks_total_size, checksum(4)
        out.extend_from_slice(&1u64.to_be_bytes());
        out.extend_from_slice(&1u64.to_be_bytes());
        out.extend_from_slice(&(kbi_wrapped_decomp_len(&kbi_wrapped) as u64).to_be_bytes());
        out.extend_from_slice(&(kbi_final.len() as u64).to_be_bytes());
        out.extend_from_slice(&(key_block_wrapped.len() as u64).to_be_bytes());
        out.extend_from_slice(&[0u8; 4]);
        out.extend_from_slice(&kbi_final);
        out.extend_from_slice(&key_block_wrapped);

        // --- record section: one record block containing the HTML ---
        let record_plain = html.as_bytes().to_vec();
        let record_comp_payload = zlib_compress(&record_plain);
        let mut record_wrapped = Vec::new();
        record_wrapped.extend_from_slice(&RECORD_BLOCK_COMP_ZLIB.to_be_bytes());
        record_wrapped.extend_from_slice(&[0u8; 4]);
        record_wrapped.extend_from_slice(&record_comp_payload);

        out.extend_from_slice(&1u64.to_be_bytes()); // num_record_blocks
        out.extend_from_slice(&1u64.to_be_bytes()); // num_entries
        out.extend_from_slice(&8u64.to_be_bytes()); // record_index_size (unused by parser)
        out.extend_from_slice(&(record_wrapped.len() as u64).to_be_bytes());
        out.extend_from_slice(&(record_wrapped.len() as u64).to_be_bytes()); // comp_sz
        out.extend_from_slice(&(record_plain.len() as u64).to_be_bytes()); // decomp_sz
        out.extend_from_slice(&record_wrapped);

        out
    }

    fn kbi_wrapped_decomp_len(kbi_wrapped: &[u8]) -> usize {
        // kbi_wrapped = [comp_type(4)][adler32(4)][zlib payload]; we need the
        // *plaintext* (pre-zlib) length, which the builder above already knows,
        // so this helper just re-derives it by decompressing.
        let decoder_payload = &kbi_wrapped[8..];
        let mut decoder = ZlibDecoder::new(decoder_payload);
        let mut out = Vec::new();
        decoder.read_to_end(&mut out).unwrap();
        out.len()
    }

    fn zlib_compress(data: &[u8]) -> Vec<u8> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    /// Test-side encryptor matching the real MDict keyword-index-encryption
    /// scheme (see decrypt_key_block_info's doc comment for the full
    /// derivation). Unlike RC4, this cipher is NOT symmetric — encryption
    /// and decryption use different formulas, because `previous` chains
    /// off the *ciphertext* byte in both directions, so this is a distinct
    /// forward pass, not a call to the decrypt function.
    fn encrypt_key_block_info_for_test(data: &[u8]) -> Vec<u8> {
        let mut key_material = [0u8; 8];
        key_material[0..4].copy_from_slice(&data[4..8]);
        key_material[4..8].copy_from_slice(&[0x95, 0x36, 0x00, 0x00]);
        let key = Ripemd128::digest(key_material);

        let mut out = data.to_vec();
        let mut previous: u8 = 0x36;
        for (i, byte) in out[8..].iter_mut().enumerate() {
            let plain_byte = *byte;
            let x = plain_byte ^ (i as u8) ^ key[i % key.len()] ^ previous;
            let cipher_byte = (x >> 4) | (x << 4); // SWAPNIBBLE
            previous = cipher_byte;
            *byte = cipher_byte;
        }
        out
    }

    fn parse_via_tempfile(bytes: &[u8]) -> Result<MdxDict> {
        let mut path = std::env::temp_dir();
        path.push(format!("wispet_mdx_test_{}.mdx", std::process::id()));
        std::fs::write(&path, bytes)?;
        let result = MdxDict::open(&path);
        let _ = std::fs::remove_file(&path);
        result
    }

    #[test]
    fn header_len_is_read_as_big_endian() {
        // header_len = 2808 encoded BE, matching the real vocabulary.com
        // and LDOCE6 header lengths observed in practice. If this were
        // misread as little-endian it would decode to billions and this
        // whole file would fail to even reach the header string.
        let bytes = build_synthetic_mdx(false, "hello", "<b>hi</b>");
        let declared_be = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        assert!(declared_be < 10_000, "header_len should be small, got {declared_be}");
    }

    #[test]
    fn parses_unencrypted_synthetic_dictionary() {
        let bytes = build_synthetic_mdx(false, "hello", "<b>hi there</b>");
        let dict = parse_via_tempfile(&bytes).expect("unencrypted MDX should parse");
        let html = dict.lookup_exact("hello").unwrap();
        assert_eq!(html.as_deref(), Some("<b>hi there</b>"));
    }

    #[test]
    fn parses_encrypted_synthetic_dictionary() {
        // Mirrors Encrypted="2" as seen in both LDOCE6.mdx and
        // vocabulary.com's MDX header.
        let bytes = build_synthetic_mdx(true, "hello", "<b>hi there</b>");
        let dict = parse_via_tempfile(&bytes).expect("encrypted MDX should parse");
        let html = dict.lookup_exact("hello").unwrap();
        assert_eq!(html.as_deref(), Some("<b>hi there</b>"));
    }

    #[test]
    fn lookup_is_case_insensitive_and_missing_word_returns_none() {
        let bytes = build_synthetic_mdx(true, "Hello", "<b>hi</b>");
        let dict = parse_via_tempfile(&bytes).unwrap();
        assert!(dict.lookup_exact("HELLO").unwrap().is_some());
        assert!(dict.lookup_exact("goodbye").unwrap().is_none());
    }

    #[test]
    fn implausible_length_field_fails_fast_instead_of_allocating() {
        // Simulates the original bug's symptom directly: a length field
        // that decodes to billions must produce a clean error, not an
        // attempted multi-gigabyte allocation or a panic.
        let err = checked_len("test_field", 50 * 1024 * 1024 * 1024).unwrap_err();
        assert!(err.to_string().contains("implausibly large"));
    }

    #[test]
    fn rejects_v1_2_instead_of_misparsing_it() {
        let header_xml = "<Dictionary GeneratedByEngineVersion=\"1.2\" Encrypted=\"0\" \
                           Format=\"Html\" Encoding=\"UTF-8\"/>\r\n";
        let header_utf16: Vec<u8> = header_xml
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .chain(std::iter::once(0u16).flat_map(|u| u.to_le_bytes()))
            .collect();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(header_utf16.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&header_utf16);
        bytes.extend_from_slice(&[0u8; 4]);

        let err = parse_via_tempfile(&bytes).unwrap_err();
        assert!(err.to_string().contains("v1.2") || err.to_string().contains("2.0"));
    }
}
