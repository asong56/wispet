use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod deepl;
pub mod google;
pub mod mdx;
pub mod wikipedia;

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
    pub phonetic: Option<String>,
    /// BCP-47 source language, if detected
    pub source_lang: Option<String>,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;

    /// `Ok(None)` means no result found; `Err` is a hard failure.
    async fn lookup(&self, query: &str) -> anyhow::Result<Option<ProviderResult>>;
}

/// Run all providers concurrently; join_all preserves priority order.
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

    join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect()
}
