//! Unofficial public Google Translate endpoint. If Google changes it,
//! disable this provider in config and use DeepL instead.

use super::{Provider, ProviderResult, ResultKind};
use anyhow::Result;
use async_trait::async_trait;

pub struct GoogleProvider {
    id: String,
    label: String,
    client: reqwest::Client,
    source_lang: String,
    target_lang: String,
}

impl GoogleProvider {
    pub fn new(id: String, label: String, source_lang: String, target_lang: String) -> Self {
        GoogleProvider {
            id,
            label,
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (compatible; Wispet/0.1)")
                .build()
                .unwrap_or_default(),
            source_lang,
            target_lang,
        }
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    fn id(&self) -> &str { &self.id }
    fn label(&self) -> &str { &self.label }

    async fn lookup(&self, query: &str) -> Result<Option<ProviderResult>> {
        if query.trim().is_empty() {
            return Ok(None);
        }

        let sl = if self.source_lang == "auto" { "auto" } else { &self.source_lang };
        let url = format!(
            "https://translate.googleapis.com/translate_a/single\
             ?client=gtx&sl={}&tl={}&dt=t&q={}",
            sl,
            self.target_lang,
            urlencoding::encode(query)
        );

        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            anyhow::bail!("Google Translate: HTTP {}", resp.status());
        }

        let raw: serde_json::Value = resp.json().await?;

        // Undocumented shape: raw[0] is per-sentence segments (["text","src","",""]),
        // raw[2] is the detected source language.
        let translation = raw.get(0).and_then(|segments| segments.as_array()).map(|segments| {
            segments
                .iter()
                .filter_map(|seg| seg.get(0).and_then(|v| v.as_str()))
                .collect::<String>()
        });

        let detected = raw
            .get(2)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match translation {
            None => Ok(None),
            Some(text) if text.trim().is_empty() => Ok(None),
            Some(text) => Ok(Some(ProviderResult {
                provider_id: self.id.clone(),
                provider_label: self.label.clone(),
                kind: ResultKind::Translation,
                content: text,
                phonetic: None,
                source_lang: detected,
            })),
        }
    }
}
