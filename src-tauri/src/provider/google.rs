//! Google Translate provider using the public (unofficial) single-token endpoint.
//! Falls back gracefully; if Google changes their API, set `enabled = false`
//! and use DeepL instead.

use super::{Provider, ProviderResult, ResultKind};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

pub struct GoogleProvider {
    client: reqwest::Client,
    source_lang: String,
    target_lang: String,
}

impl GoogleProvider {
    pub fn new(source_lang: String, target_lang: String) -> Self {
        GoogleProvider {
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
    fn id(&self) -> &str { "google" }
    fn label(&self) -> &str { "Google Translate" }

    async fn lookup(&self, query: &str) -> Result<Option<ProviderResult>> {
        if query.trim().is_empty() {
            return Ok(None);
        }

        // Google Translate unofficial single-sentence endpoint
        // Returns a nested JSON array; position [0][0][0] is the translation,
        // [0][0][1] is the source text, [2] is the detected source language.
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

        // Parse nested array: [[["translation","source","","",""],...],...,"detected_lang",...]
        let translation = raw
            .get(0)
            .and_then(|a| a.get(0))
            .and_then(|a| a.get(0))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let detected = raw
            .get(2)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match translation {
            None => Ok(None),
            Some(text) if text.trim().is_empty() => Ok(None),
            Some(text) => Ok(Some(ProviderResult {
                provider_id: "google".to_string(),
                provider_label: "Google Translate".to_string(),
                kind: ResultKind::Translation,
                content: text,
                phonetic: None,
                source_lang: detected,
            })),
        }
    }
}
