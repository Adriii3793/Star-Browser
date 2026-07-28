use std::sync::OnceLock;
use std::time::Duration;
use serde::Serialize;
use crate::error::AppError;

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_TEXT_CHARS: usize = 16_000;
const MAX_IMAGES: usize = 12;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageContext {
    pub url: String,
    pub title: String,
    pub text: String,
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub truncated: bool,
}

fn client() -> Result<&'static reqwest::Client, AppError> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(8))
             .user_agent("Mozilla/5.0 (compatible; star-browser/0.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .build()
            .ok()
    }).as_ref().ok_or(AppError::PageFetch)
}

#[tauri::command]
pub async fn fetch_page_context(url: String) -> Result<PageContext, AppError> {
    let parsed = url::Url::parse(&url).map_err(|_| AppError::InvalidUrl)?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::InvalidUrl);
    }

    let mut response = client()?.get(parsed).send().await.map_err(|_| AppError::PageFetch)?;
    if !response.status().is_success() {
        return Err(AppError::PageFetch);
    }

    let mut body: Vec<u8> = Vec::new();
    let mut hit_cap = false;
    while let Some(chunk) = response.chunk().await.map_err(|_| AppError::PageFetch)? {
        let room = MAX_BYTES.saturating_sub(body.len());
        if room == 0 {
            hit_cap = true;
            break;
        }
        let take = room.min(chunk.len());
        body.extend_from_slice(&chunk[..take]);
        if take < chunk.len() {
            hit_cap = true;
            break;
        }
    }

    let html = String::from_utf8_lossy(&body).into_owned();
    let final_url = response.url().clone();

    let title = between(&html, "<title", "</title>")
        .map(|t| t.trim_start_matches('>').trim().chars().take(300).collect())
        .unwrap_or_default();

    let images = sources(&html, "<img", &final_url, MAX_IMAGES);
    let videos = sources(&html, "<source", &final_url, 6);

    let full = visible_text(&html);
    let mut text: String = full.chars().take(MAX_TEXT_CHARS).collect();
    let truncated = hit_cap || text.chars().count() < full.chars().count();
    if truncated {
        text.push_str("\n[content truncated]");
    }

    Ok(PageContext { url: final_url.to_string(), title, text, images, videos, truncated })
}

fn between(html: &str, open: &str, close: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let s = lower.find(open)? + open.len();
    let e = lower[s..].find(close)? + s;
    Some(html[s..e].to_string())
}

fn remove_blocks(html: &str, tag: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let (open, close) = (format!("<{tag}"), format!("</{tag}>"));
    let (mut out, mut cur) = (String::with_capacity(html.len()), 0usize);
    while let Some(rel) = lower[cur..].find(&open) {
        let start = cur + rel;
        out.push_str(&html[cur..start]);
        match lower[start..].find(&close) {
            Some(e) => cur = start + e + close.len(),
            None => return out,
        }
    }
    out.push_str(&html[cur..]);
    out
}

fn sources(html: &str, tag: &str, base: &url::Url, limit: usize) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let (mut out, mut cur) = (Vec::new(), 0usize);
    while let Some(rel) = lower[cur..].find(tag) {
        let start = cur + rel;
        let end = match lower[start..].find('>') { Some(e) => start + e, None => break };
        if let Some(src) = attr(&html[start..end], "src") {
            if let Ok(abs) = base.join(&src) {
                if matches!(abs.scheme(), "http" | "https") {
                    let abs = abs.to_string();
                    if !out.contains(&abs) {
                        out.push(abs);
                        if out.len() >= limit { return out; }
                    }
                }
            }
        }
        cur = end + 1;
    }
    out
}

fn attr(tag_src: &str, name: &str) -> Option<String> {
    let lower = tag_src.to_ascii_lowercase();
    let at = lower.find(name)?;
    let rest = tag_src[at + name.len()..].trim_start().strip_prefix('=')?.trim_start();
    match rest.chars().next()? {
        q @ ('"' | '\'') => rest[1..].find(q).map(|e| rest[1..1 + e].to_string()),
        _ => Some(rest.split_whitespace().next()?.to_string()),
    }
}

fn visible_text(html: &str) -> String {
    let c = remove_blocks(html, "script");
    let c = remove_blocks(&c, "style");
    let c = remove_blocks(&c, "noscript");
    let c = remove_blocks(&c, "head");

    let mut text = String::with_capacity(c.len() / 2);
    let mut in_tag = false;
    for ch in c.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => { in_tag = false; text.push(' '); }
            x if !in_tag => text.push(x),
            _ => {}
        }
    }
    text.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
        .replace("&quot;", "\"").replace("&#39;", "'").replace("&nbsp;", " ")
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
