use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod deepl;
pub mod google;
pub mod mdx;
pub mod wikipedia;

// ── Result types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    /// Structured dictionary entry (HTML)
    Dict,
    /// Machine translation (plain text)
    Translation,
    /// Encyclopedia article excerpt (HTML)
    Article,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResult {
    pub provider_id: String,
    pub provider_label: String,
    pub kind: ResultKind,
    /// Sanitized HTML for Dict/Article, or plain UTF-8 for Translation
    pub content: String,
    /// IPA or pronunciation hint if available
    pub phonetic: Option<String>,
    /// Source language detected (BCP-47)
    pub source_lang: Option<String>,
}

// ── Provider trait ────────────────────────────────────────────────────────────

#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;

    /// Perform a lookup/translation for `query`.
    /// Returns `None` if the provider has no result (no entry found, empty
    /// response, etc.) and `Err` only for hard failures (network error, bad
    /// API key, etc.).
    async fn lookup(&self, query: &str) -> anyhow::Result<Option<ProviderResult>>;
}

// ── Dispatcher ────────────────────────────────────────────────────────────────

/// Run all providers concurrently and collect results in priority order.
pub async fn dispatch(
    providers: &[Box<dyn Provider>],
    query: &str,
) -> Vec<ProviderResult> {
    use futures::future::join_all;

    let futures: Vec<_> = providers
        .iter()
        .map(|p| async move {
            match p.lookup(query).await {
                Ok(Some(r)) => Some(r),
                Ok(None) => None,
                Err(e) => {
                    log::warn!("[{}] lookup error: {:#}", p.id(), e);
                    None
                }
            }
        })
        .collect();

    // join_all preserves the original order (= priority order from config)
    join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect()
}
