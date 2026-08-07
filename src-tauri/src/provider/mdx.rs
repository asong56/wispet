use super::{Provider, ProviderResult, ResultKind};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use encoding_rs::UTF_16LE;
use flate2::read::ZlibDecoder;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

// ── MDX file constants ────────────────────────────────────────────────────────

#[allow(dead_code)]
const MDX_MAGIC: &[u8] = b"MDict dictionary file\x00";
const RECORD_BLOCK_COMP_NONE: u32 = 0x00000000;
const RECORD_BLOCK_COMP_LZO: u32 = 0x01000000;
const RECORD_BLOCK_COMP_ZLIB: u32 = 0x02000000;

// ── Key index entry ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct KeyEntry {
    /// Headword in lowercase for case-insensitive lookup
    headword_lower: String,
    /// Original headword (display)
    headword: String,
    /// Byte offset in the record blocks section for this entry's content
    record_start: u64,
    record_end: u64,
}

// ── MdxDict ───────────────────────────────────────────────────────────────────

pub struct MdxDict {
    path: PathBuf,
    /// Sorted by headword_lower for binary search
    index: Vec<KeyEntry>,
    /// (block_offset, block_compressed_size, block_decompressed_size)
    record_blocks: Vec<(u64, u64, u64)>,
    /// Byte offset in the file where record blocks begin
    record_section_offset: u64,
}

impl MdxDict {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)
            .with_context(|| format!("Cannot open MDX file: {}", path.display()))?;
        let mut reader = BufReader::new(file);

        let (index, record_blocks, record_section_offset) =
            parse_mdx(&mut reader, &path)?;

        Ok(MdxDict {
            path,
            index,
            record_blocks,
            record_section_offset,
        })
    }

    /// Look up an exact headword (case-insensitive).
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

                let mut comp_buf = vec![0u8; *blk_comp_sz as usize];
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

// ── MDX parsing ──────────────────────────────────────────────────────────────

fn parse_mdx(
    reader: &mut BufReader<File>,
    _path: &Path,
) -> Result<(Vec<KeyEntry>, Vec<(u64, u64, u64)>, u64)> {
    let header_len = read_u32_le(reader)?;
    let mut header_bytes = vec![0u8; header_len as usize];
    reader.read_exact(&mut header_bytes)?;
    let mut _adler = [0u8; 4];
    reader.read_exact(&mut _adler)?;

    let header_str = String::from_utf16_le_lossy(&header_bytes);
    let _version: f32 = parse_header_attr(&header_str, "GeneratedByEngineVersion")
        .unwrap_or("2.0".to_string())
        .parse()
        .unwrap_or(2.0);

    // MDX v2 uses 8-byte counts
    let num_blocks = read_u64_be(reader)?;
    let _num_entries = read_u64_be(reader)?;
    let key_index_decomp_size = read_u64_be(reader)?;
    let key_index_comp_size = read_u64_be(reader)?;
    let key_blocks_total_size = read_u64_be(reader)?;
    let mut _ck = [0u8; 4];
    reader.read_exact(&mut _ck)?;

    let mut key_index_comp = vec![0u8; key_index_comp_size as usize];
    reader.read_exact(&mut key_index_comp)?;

    // Key block info entries (40 bytes in v2): num_entries u64, first_headword_size
    // u16, first_headword UTF-16, last_headword_size u16, last_headword UTF-16,
    // comp_size u64, decomp_size u64 — only comp/decomp sizes are needed here.
    let key_index_decomp = decompress_block(&key_index_comp, key_index_decomp_size)?;
    let key_block_sizes = parse_key_block_info(&key_index_decomp, num_blocks)?;

    let mut index: Vec<KeyEntry> = Vec::new();
    {
        let key_blocks_start = reader.stream_position()?;
        for (comp_sz, decomp_sz) in &key_block_sizes {
            let mut comp = vec![0u8; *comp_sz as usize];
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

    // (file_relative_offset, comp_sz, decomp_sz) per block
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

    // Patch record_end for every entry now that we have the full offset picture.
    fill_record_ends(&mut index);

    Ok((index, record_blocks, record_section_offset))
}

fn parse_key_block_info(data: &[u8], num_blocks: u64) -> Result<Vec<(u64, u64)>> {
    let mut sizes: Vec<(u64, u64)> = Vec::with_capacity(num_blocks as usize);
    let mut pos = 0usize;

    for _ in 0..num_blocks {
        // num_entries (8) + first_size (2) + first_word (varies) + last_size (2) + last_word (varies)
        // + comp_size (8) + decomp_size (8)
        if pos + 8 > data.len() { break; }
        pos += 8; // skip num_entries
        let first_sz = u16::from_be_bytes([data[pos], data[pos+1]]) as usize;
        pos += 2 + (first_sz + 1) * 2; // UTF-16 chars + null terminator
        let last_sz = u16::from_be_bytes([data[pos], data[pos+1]]) as usize;
        pos += 2 + (last_sz + 1) * 2;
        let comp_sz = u64::from_be_bytes(data[pos..pos+8].try_into()?);
        pos += 8;
        let decomp_sz = u64::from_be_bytes(data[pos..pos+8].try_into()?);
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

        // Null-terminated UTF-16LE headword
        let mut chars: Vec<u16> = Vec::new();
        while pos + 2 <= data.len() {
            let ch = u16::from_le_bytes([data[pos], data[pos+1]]);
            pos += 2;
            if ch == 0 { break; }
            chars.push(ch);
        }

        let headword = String::from_utf16_lossy(&chars).to_string();

        // record_end will be filled in a second pass below
        index.push(KeyEntry {
            headword_lower: headword.to_lowercase(),
            headword,
            record_start: record_offset,
            record_end: 0,
        });
    }

    Ok(())
}

/// After all key blocks have been collected and sorted by headword, patch each
/// entry's record_end by sorting a separate index by record_start and chaining
/// adjacent offsets. The last entry gets u64::MAX as a sentinel; read_record
/// clamps to decompressed.len() so it is safe.
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
        // A valid MDX block always has at least a 4-byte type tag and 4-byte
        // checksum before any payload.  Anything shorter is corrupt data.
        anyhow::bail!("MDX block too short to contain a compression header ({} bytes)", data.len());
    }
    let comp_type = u32::from_be_bytes(data[0..4].try_into()?);
    // Bytes 4..8 are adler32 checksum — skip
    let payload = &data[8..];

    match comp_type {
        RECORD_BLOCK_COMP_NONE => Ok(payload.to_vec()),
        RECORD_BLOCK_COMP_LZO => {
            // LZO1X decompression is not self-terminating like zlib — the
            // exact output size must be known ahead of time (it's given by
            // the block's decomp_sz field in the key/record block index).
            let mut out = vec![0u8; decompressed_size as usize];
            lzo1x::decompress(payload, &mut out)
                .map_err(|e| anyhow::anyhow!("LZO decompression failed: {:?}", e))?;
            Ok(out)
        }
        RECORD_BLOCK_COMP_ZLIB => {
            let mut decoder = ZlibDecoder::new(payload);
            let mut out = Vec::with_capacity(decompressed_size as usize);
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        other => {
            bail!("Unsupported MDX compression type: 0x{:08X}", other)
        }
    }
}

fn decode_content(bytes: &[u8]) -> String {
    // MDX v2: UTF-16LE null-terminated
    if bytes.len() >= 2 && bytes[1] == 0 {
        let (decoded, _, _) = UTF_16LE.decode(bytes);
        decoded.trim_end_matches('\0').to_string()
    } else {
        // v1 fallback: UTF-8 / GB18030
        String::from_utf8_lossy(bytes)
            .trim_end_matches('\0')
            .to_string()
    }
}

// ── Low-level read helpers ────────────────────────────────────────────────────

fn read_u32_le(r: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
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

// Lossy UTF-16 LE decode for header parsing only
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

// ── MDD (media resource) support ────────────────────────────────────────────
// Same container format as MDX (see fileformat.md), so `parse_mdx` is reused
// verbatim; only the semantics differ — keys are resource paths and records
// are raw bytes rather than UTF-16 HTML text.

/// Reads an MDD (MDict resource container) file: pronunciation audio,
/// images, and stylesheets bundled alongside an MDX dictionary.
pub struct MddDict {
    path: PathBuf,
    /// Sorted by (lowercased) resource path for binary search — reuses
    /// KeyEntry from the MDX index; `headword` holds the resource path.
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

    /// Look up a resource by the path referenced from MDX HTML (e.g.
    /// `word.mp3` from a rewritten `sound://word.mp3` link). MDD entries are
    /// conventionally stored with a leading backslash and backslash
    /// separators (Windows-style, regardless of the platform that built the
    /// dictionary) — since the exact convention varies between dictionaries,
    /// a few common variants are tried.
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

                let mut comp_buf = vec![0u8; *blk_comp_sz as usize];
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

// ── Provider impl ─────────────────────────────────────────────────────────────

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
