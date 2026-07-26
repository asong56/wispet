use super::{Provider, ProviderResult, ResultKind};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct DeeplProvider {
    client: reqwest::Client,
    api_key: String,
    source_lang: String,
    target_lang: String,
}

impl DeeplProvider {
    pub fn new(api_key: String, source_lang: String, target_lang: String) -> Self {
        DeeplProvider {
            client: reqwest::Client::new(),
            api_key,
            source_lang,
            target_lang,
        }
    }

    fn is_free_key(&self) -> bool {
        self.api_key.ends_with(":fx")
    }

    fn api_url(&self) -> &'static str {
        if self.is_free_key() {
            "https://api-free.deepl.com/v2/translate"
        } else {
            "https://api.deepl.com/v2/translate"
        }
    }
}

#[derive(Serialize)]
struct DeeplRequest<'a> {
    text: Vec<&'a str>,
    target_lang: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_lang: Option<&'a str>,
}

#[derive(Deserialize)]
struct DeeplResponse {
    translations: Vec<DeeplTranslation>,
}

#[derive(Deserialize)]
struct DeeplTranslation {
    text: String,
    detected_source_language: String,
}

#[async_trait]
impl Provider for DeeplProvider {
    fn id(&self) -> &str { "deepl" }
    fn label(&self) -> &str { "DeepL" }

    async fn lookup(&self, query: &str) -> Result<Option<ProviderResult>> {
        if query.trim().is_empty() {
            return Ok(None);
        }

        let source = if self.source_lang == "auto" {
            None
        } else {
            Some(self.source_lang.as_str())
        };

        let body = DeeplRequest {
            text: vec![query],
            target_lang: &self.target_lang,
            source_lang: source,
        };

        let resp = self
            .client
            .post(self.api_url())
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if resp.status() == 403 {
            bail!("DeepL: invalid API key");
        }
        if resp.status() == 456 {
            bail!("DeepL: quota exceeded");
        }
        if !resp.status().is_success() {
            bail!("DeepL API error: {}", resp.status());
        }

        let data: DeeplResponse = resp.json().await?;
        let first = data.translations.into_iter().next();

        match first {
            None => Ok(None),
            Some(t) => Ok(Some(ProviderResult {
                provider_id: "deepl".to_string(),
                provider_label: "DeepL".to_string(),
                kind: ResultKind::Translation,
                content: t.text,
                phonetic: None,
                source_lang: Some(t.detected_source_language),
            })),
        }
    }
}
