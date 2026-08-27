use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::error::AppError;

pub fn unique_download_path(app: &AppHandle, file_name: &str) -> PathBuf {
    let dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().home_dir())
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&dir);

    let base = Path::new(file_name);
    let stem = base
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("download")
        .to_string();
    let ext = base
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();

    let mut candidate = dir.join(format!("{stem}{ext}"));
    let mut n = 1u32;
    while candidate.exists() {
        candidate = dir.join(format!("{stem} ({n}){ext}"));
        n += 1;
    }
    candidate
}

fn safe_file_name(raw: &str, fallback: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

#[tauri::command]
pub async fn save_text_file(
    app: AppHandle,
    file_name: String,
    contents: String,
) -> Result<String, AppError> {
    let name = safe_file_name(&file_name, "star-export.md");
    let target = unique_download_path(&app, &name);
    std::fs::write(&target, contents)?;
    Ok(target.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::safe_file_name;

    #[test]
    fn strips_path_separators_so_a_title_cannot_escape_downloads() {
        assert_eq!(safe_file_name("a/b\\c", "fallback"), "a-b-c");

        for hostile in [
            "../../etc/passwd",
            "..\\..\\Windows\\System32\\config",
            "/absolute/path",
            "C:\\Windows\\evil",
        ] {
            let safe = safe_file_name(hostile, "fallback");
            assert!(!safe.contains('/'), "{safe:?} still contains a forward slash");
            assert!(!safe.contains('\\'), "{safe:?} still contains a backslash");
            assert!(!safe.contains(':'), "{safe:?} still contains a drive separator");
            assert_eq!(
                std::path::Path::new(&safe).components().count(),
                1,
                "{safe:?} is not a single path component"
            );
        }
    }

    #[test]
    fn falls_back_when_nothing_usable_is_left() {
        assert_eq!(safe_file_name("   ", "fallback"), "fallback");
        assert_eq!(safe_file_name("...", "fallback"), "fallback");
    }

    #[test]
    fn keeps_ordinary_names_intact() {
        assert_eq!(safe_file_name("star-chat.md", "fallback"), "star-chat.md");
    }
}
