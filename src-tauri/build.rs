use std::path::Path;

fn warn_embedded() {
    println!("cargo:warning==================================================================");
    println!("cargo:warning=OPENROUTER_API_KEY IS BEING COMPILED INTO THIS BINARY IN PLAIN TEXT.");
    println!("cargo:warning=Anyone with the binary can read it: `strings star.exe | grep sk-or-`.");
    println!(
        "cargo:warning=Treat any key shipped this way as public, and never use a personal one."
    );
    println!("cargo:warning=Prefer STAR_AI_PROXY (see docs/ai-proxy/README.md), which keeps the");
    println!("cargo:warning=key on a server you control so it never reaches the binary at all.");
    println!("cargo:warning==================================================================");
}

fn embed_proxy_url() {
    println!("cargo:rerun-if-env-changed=STAR_AI_PROXY");
    if let Ok(url) = std::env::var("STAR_AI_PROXY") {
        let url = url.trim();
        if !url.is_empty() {
            println!("cargo:rustc-env=STAR_AI_PROXY={url}");
        }
    }
}

fn env_file_value(name: &str) -> Option<String> {
    for candidate in ["../.env", ".env"] {
        let Ok(contents) = std::fs::read_to_string(Path::new(candidate)) else {
            continue;
        };
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() != name {
                continue;
            }
            let value = value.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

fn embedding_requested() -> bool {
    println!("cargo:rerun-if-env-changed=STAR_EMBED_API_KEY");
    let choice = std::env::var("STAR_EMBED_API_KEY")
        .ok()
        .or_else(|| env_file_value("STAR_EMBED_API_KEY"));
    match choice {
        Some(value) => !matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "no" | "off"
        ),
        None => true,
    }
}

fn embed_api_key() {
    println!("cargo:rerun-if-env-changed=OPENROUTER_API_KEY");
    println!("cargo:rerun-if-changed=../.env");
    println!("cargo:rerun-if-changed=.env");

    if !embedding_requested() {
        return;
    }

    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        if !key.trim().is_empty() {
            println!("cargo:rustc-env=STAR_EMBEDDED_API_KEY={}", key.trim());
            warn_embedded();
            return;
        }
    }

    if let Some(key) = env_file_value("OPENROUTER_API_KEY") {
        println!("cargo:rustc-env=STAR_EMBEDDED_API_KEY={key}");
        warn_embedded();
    }
}

fn emit_gtk_cfg() {
    println!("cargo:rustc-check-cfg=cfg(gtk)");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if matches!(
        os.as_str(),
        "linux" | "dragonfly" | "freebsd" | "netbsd" | "openbsd"
    ) {
        println!("cargo:rustc-cfg=gtk");
    }
}

fn main() {
    emit_gtk_cfg();
    embed_proxy_url();
    embed_api_key();
    tauri_build::build()
}
