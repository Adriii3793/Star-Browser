use std::sync::OnceLock;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_TEXT_CHARS: usize = 16_000;
const MAX_IMAGES: usize = 12;

#[derive(Serialize, Deserialize)]
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
    let bytes = lower.as_bytes();
    let mut from = 0usize;

    while let Some(rel) = lower[from..].find(name) {
        let at = from + rel;
        from = at + name.len();

        let starts_attribute = at > 0 && bytes[at - 1].is_ascii_whitespace();
        if !starts_attribute {
            continue;
        }
        let after = &tag_src[at + name.len()..];
        let Some(rest) = after.trim_start().strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim_start();
        return match rest.chars().next()? {
            q @ ('"' | '\'') => rest[1..].find(q).map(|e| rest[1..1 + e].to_string()),
            _ => rest.split_whitespace().next().map(str::to_string),
        };
    }
    None
}

fn decode_entities(input: &str) -> String {
    const NAMED: &[(&str, &str)] = &[
        ("amp", "&"), ("lt", "<"), ("gt", ">"), ("quot", "\""), ("apos", "'"),
        ("nbsp", " "), ("mdash", "\u{2014}"), ("ndash", "\u{2013}"), ("hellip", "\u{2026}"),
        ("lsquo", "\u{2018}"), ("rsquo", "\u{2019}"), ("ldquo", "\u{201C}"), ("rdquo", "\u{201D}"),
        ("laquo", "\u{00AB}"), ("raquo", "\u{00BB}"), ("times", "\u{00D7}"), ("middot", "\u{00B7}"),
        ("bull", "\u{2022}"), ("deg", "\u{00B0}"), ("euro", "\u{20AC}"), ("pound", "\u{00A3}"),
        ("copy", "\u{00A9}"), ("reg", "\u{00AE}"), ("trade", "\u{2122}"), ("shy", ""),
    ];

    if !input.contains('&') {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp..];
        let Some(semi) = tail
            .char_indices()
            .take(12)
            .find(|(_, c)| *c == ';')
            .map(|(i, _)| i)
        else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let body = &tail[1..semi];
        let decoded = if let Some(hex) = body.strip_prefix("#x").or_else(|| body.strip_prefix("#X")) {
            u32::from_str_radix(hex, 16).ok().and_then(char::from_u32).map(String::from)
        } else if let Some(dec) = body.strip_prefix('#') {
            dec.parse::<u32>().ok().and_then(char::from_u32).map(String::from)
        } else {
            NAMED
                .iter()
                .find(|(name, _)| *name == body)
                .map(|(_, value)| (*value).to_string())
        };

        match decoded {
            Some(value) => {
                out.push_str(&value);
                rest = &tail[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
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
    decode_entities(&text)
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{attr, decode_entities};

    #[test]
    fn reads_src_even_when_data_src_or_srcset_comes_first() {
        assert_eq!(
            attr(r#"<img data-src="lazy.jpg" src="real.jpg""#, "src").as_deref(),
            Some("real.jpg")
        );
        assert_eq!(
            attr(r#"<img srcset="a.jpg 1x, b.jpg 2x" src="real.jpg""#, "src").as_deref(),
            Some("real.jpg")
        );
        assert_eq!(
            attr(r#"<img class="hero-src" src='real.jpg'"#, "src").as_deref(),
            Some("real.jpg")
        );
    }

    #[test]
    fn handles_unquoted_and_missing_values() {
        assert_eq!(attr("<img src=plain.jpg alt=x", "src").as_deref(), Some("plain.jpg"));
        assert_eq!(attr(r#"<img data-src="only-lazy.jpg""#, "src"), None);
        assert_eq!(attr("<img>", "src"), None);
    }

    #[test]
    fn tolerates_whitespace_around_the_equals_sign() {
        assert_eq!(attr(r#"<img  src = "spaced.jpg""#, "src").as_deref(), Some("spaced.jpg"));
    }

    #[test]
    fn decodes_named_numeric_and_hex_entities() {
        assert_eq!(decode_entities("a &amp; b"), "a & b");
        assert_eq!(decode_entities("it&#39;s"), "it's");
        assert_eq!(decode_entities("it&#8217;s"), "it\u{2019}s");
        assert_eq!(decode_entities("&#x2014;"), "\u{2014}");
        assert_eq!(decode_entities("&mdash;&hellip;"), "\u{2014}\u{2026}");
    }

    #[test]
    fn leaves_stray_ampersands_and_unknown_names_alone() {
        assert_eq!(decode_entities("Tom & Jerry"), "Tom & Jerry");
        assert_eq!(decode_entities("a=1&b=2"), "a=1&b=2");
        assert_eq!(decode_entities("caf&eacute;"), "caf&eacute;");
        assert_eq!(decode_entities("no entities here"), "no entities here");
    }

    #[test]
    fn does_not_split_multibyte_characters_after_an_ampersand() {
        assert_eq!(decode_entities("&日本語のテキスト"), "&日本語のテキスト");
        assert_eq!(decode_entities("a & 日本語 &amp; b"), "a & 日本語 & b");
        assert_eq!(decode_entities("&🎉🎉🎉🎉;"), "&🎉🎉🎉🎉;");
    }
}
