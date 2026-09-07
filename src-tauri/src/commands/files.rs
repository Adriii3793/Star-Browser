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

const RESERVED_STEMS: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

fn is_reserved(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).trim_end();
    RESERVED_STEMS.iter().any(|r| stem.eq_ignore_ascii_case(r))
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
        return fallback.to_string();
    }
    let capped: String = trimmed.chars().take(120).collect();
    if is_reserved(&capped) {
        format!("_{capped}")
    } else {
        capped
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
            assert!(
                !safe.contains('/'),
                "{safe:?} still contains a forward slash"
            );
            assert!(!safe.contains('\\'), "{safe:?} still contains a backslash");
            assert!(
                !safe.contains(':'),
                "{safe:?} still contains a drive separator"
            );
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

    #[test]
    fn escapes_windows_device_names_so_the_export_is_not_swallowed() {
        for reserved in ["NUL", "nul.md", "CON.txt", "com1", "LPT9.md", "Aux"] {
            let safe = safe_file_name(reserved, "fallback");
            assert!(
                safe.starts_with('_'),
                "{reserved:?} produced {safe:?}, which Windows still treats as a device"
            );
        }
        assert_eq!(safe_file_name("console.md", "fallback"), "console.md");
        assert_eq!(safe_file_name("nulls.md", "fallback"), "nulls.md");
    }
}
