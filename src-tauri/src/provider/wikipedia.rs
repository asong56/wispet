use super::{Provider, ProviderResult, ResultKind};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

pub struct WikipediaProvider {
    id: String,
    label: String,
    client: reqwest::Client,
    lang: String,
}

impl WikipediaProvider {
    pub fn new(id: String, label: String, lang: String) -> Self {
        WikipediaProvider {
            id,
            label,
            client: reqwest::Client::builder()
                .user_agent("Wispet/0.1 (https://github.com/wispet/wispet)")
                .build()
                .unwrap_or_default(),
            lang,
        }
    }
}

#[derive(Deserialize)]
struct WikiSummary {
    #[allow(dead_code)]
    title: String,
    extract: String,
    #[serde(rename = "extract_html")]
    extract_html: Option<String>,
    description: Option<String>,
}

#[async_trait]
impl Provider for WikipediaProvider {
    fn id(&self) -> &str { &self.id }
    fn label(&self) -> &str { &self.label }

    async fn lookup(&self, query: &str) -> Result<Option<ProviderResult>> {
        if query.trim().is_empty() {
            return Ok(None);
        }

        let encoded = urlencoding::encode(query);
        let url = format!(
            "https://{lang}.wikipedia.org/api/rest_v1/page/summary/{title}",
            lang = self.lang,
            title = encoded,
        );

        let resp = self.client.get(&url).send().await?;

        if resp.status() == 404 {
            return Ok(None);
        }

        if !resp.status().is_success() {
            anyhow::bail!("Wikipedia: HTTP {}", resp.status());
        }

        let data: WikiSummary = resp.json().await?;

        if data.extract.trim().is_empty() {
            return Ok(None);
        }

        // Prefer the API's own HTML extract; fall back to wrapping plain text.
        let html = data
            .extract_html
            .unwrap_or_else(|| format!("<p>{}</p>", html_escape(&data.extract)));

        let description = data.description.as_deref().unwrap_or("").to_string();

        let content = format!(
            r#"<p class="wiki-desc">{desc}</p>{body}"#,
            desc = html_escape(&description),
            body = html
        );

        Ok(Some(ProviderResult {
            provider_id: self.id.clone(),
            provider_label: self.label.clone(),
            kind: ResultKind::Article,
            content,
            phonetic: None,
            source_lang: Some(self.lang.clone()),
        }))
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
