use anyhow::Context;
use axum::{extract::State, response::Html};

use super::ServeHtmlConf;

impl ServeHtmlConf {
    pub fn new(prefix: &str, path: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            path: path.to_string(),
        }
    }
}

pub async fn serve_html(State(conf): State<ServeHtmlConf>) -> crate::Result<Html<String>> {
    let content = tokio::fs::read_to_string(&conf.path)
        .await
        .context("failed to open and read file")?
        .replace("{{prefix}}", &conf.prefix);
    Ok(Html(content))
}
