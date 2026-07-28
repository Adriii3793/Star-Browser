use std::path::Path;

/// Reads OPENROUTER_API_KEY at build time and bakes it into the binary.
///
/// The app used to rely on `dotenvy::from_filename("../.env")` at runtime, which only
/// resolves while running from `src-tauri/` during development. An installed build has a
/// different working directory and no `.env` beside it, so the key was always missing and
/// the AI panel reported "API key not configured".
///
/// Precedence: an already-exported env var wins, otherwise `.env` / `../.env` is parsed.
/// If neither exists the build still succeeds and the app falls back to the runtime env.
fn embed_api_key() {
    // Rerun whenever the key or either env file changes.
    println!("cargo:rerun-if-env-changed=OPENROUTER_API_KEY");
    println!("cargo:rerun-if-changed=../.env");
    println!("cargo:rerun-if-changed=.env");

    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        if !key.trim().is_empty() {
            println!("cargo:rustc-env=STAR_EMBEDDED_API_KEY={}", key.trim());
            return;
        }
    }

    for candidate in ["../.env", ".env"] {
        let path = Path::new(candidate);
        if !path.exists() {
            continue;
        }
        let Ok(contents) = std::fs::read_to_string(path) else {
            continue;
        };
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((name, value)) = line.split_once('=') else {
                continue;
            };
            if name.trim() != "OPENROUTER_API_KEY" {
                continue;
            }
            // Strip optional surrounding quotes.
            let value = value.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                println!("cargo:rustc-env=STAR_EMBEDDED_API_KEY={value}");
                return;
            }
        }
    }
}

fn main() {
    embed_api_key();
    tauri_build::build()
}
