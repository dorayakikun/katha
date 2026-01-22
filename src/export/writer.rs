use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::Session;
use crate::error::KathaError;

use super::ExportFormat;

/// 一意のファイルパスを生成（既存ファイルがある場合は連番を付与）
fn unique_path(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }

    let stem = base_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let extension = base_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let parent = base_path.parent().unwrap_or(Path::new("."));

    for i in 1..=999 {
        let new_name = if extension.is_empty() {
            format!("{}_{}", stem, i)
        } else {
            format!("{}_{}.{}", stem, i, extension)
        };
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
    }

    // 999回試しても見つからない場合はタイムスタンプを使用
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let new_name = if extension.is_empty() {
        format!("{}_{}", stem, timestamp)
    } else {
        format!("{}_{}.{}", stem, timestamp, extension)
    };
    parent.join(new_name)
}

/// ファイルに書き込み
pub fn write_to_file(
    content: &str,
    filename: &str,
    directory: Option<&Path>,
) -> Result<PathBuf, KathaError> {
    let dir = directory.unwrap_or(Path::new("."));

    // ディレクトリの存在確認
    if !dir.exists() {
        return Err(KathaError::ExportError(format!(
            "出力ディレクトリが存在しません: {}",
            dir.display()
        )));
    }

    // ディレクトリの書き込み権限確認
    let metadata = fs::metadata(dir).map_err(|e| {
        KathaError::ExportError(format!(
            "ディレクトリ情報の取得に失敗: {}: {}",
            dir.display(),
            e
        ))
    })?;

    if metadata.permissions().readonly() {
        return Err(KathaError::ExportError(format!(
            "書き込み権限がありません: {}",
            dir.display()
        )));
    }

    let base_path = dir.join(filename);

    // 既存ファイルがある場合は連番を付与
    let path = unique_path(&base_path);

    // 絶対パスに変換
    let absolute_path = fs::canonicalize(dir)
        .map(|p| p.join(path.file_name().unwrap_or_default()))
        .unwrap_or_else(|_| path.clone());

    fs::write(&path, content).map_err(|e| {
        KathaError::ExportError(format!("ファイル書き込みに失敗: {}: {}", path.display(), e))
    })?;

    Ok(absolute_path)
}

/// ファイル名を生成
pub fn generate_filename(session: &Session, format: ExportFormat) -> String {
    let project_name = session
        .project_name()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    let date_str = session
        .started_at
        .map(|dt| dt.format("%Y%m%d_%H%M").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // セッションIDの最初の8文字
    let session_id_short = session.id.chars().take(8).collect::<String>();

    format!(
        "{}_{}_{}.{}",
        project_name,
        date_str,
        session_id_short,
        format.extension()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::SessionEntry;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_session() -> Session {
        let entries = vec![SessionEntry {
            entry_type: Some("user".to_string()),
            timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            ..Default::default()
        }];

        Session::from_entries(
            "abc12345-6789".to_string(),
            "/test/my_project".to_string(),
            entries,
        )
    }

    #[test]
    fn test_generate_filename_markdown() {
        let session = create_test_session();
        let filename = generate_filename(&session, ExportFormat::Markdown);

        assert!(filename.starts_with("my_project_"));
        assert!(filename.contains("abc12345"));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn test_generate_filename_json() {
        let session = create_test_session();
        let filename = generate_filename(&session, ExportFormat::Json);

        assert!(filename.ends_with(".json"));
    }

    #[test]
    fn test_write_to_file() {
        let dir = tempdir().unwrap();
        let content = "Test content";
        let filename = "test_export.md";

        let result = write_to_file(content, filename, Some(dir.path()));
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());

        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(written, content);
    }

    #[test]
    fn test_write_to_file_current_dir() {
        let dir = tempdir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let content = "Test content";
        let filename = "test_export.md";

        let result = write_to_file(content, filename, None);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_generate_filename_with_special_chars() {
        let entries = vec![SessionEntry {
            entry_type: Some("user".to_string()),
            timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            ..Default::default()
        }];

        // 日本語やスペースを含むプロジェクト名
        let session = Session::from_entries(
            "abc12345-6789".to_string(),
            "/test/my project@123".to_string(),
            entries,
        );
        let filename = generate_filename(&session, ExportFormat::Markdown);

        // 特殊文字がアンダースコアに置換されていることを確認
        assert!(filename.starts_with("my_project_123_"));
        assert!(filename.ends_with(".md"));
        // スペースや@が含まれていないことを確認
        assert!(!filename.contains(' '));
        assert!(!filename.contains('@'));
    }

    #[test]
    fn test_write_to_file_with_existing_file() {
        let dir = tempdir().unwrap();
        let content = "Test content";
        let filename = "test_export.md";

        // 最初のファイルを作成
        let result1 = write_to_file(content, filename, Some(dir.path()));
        assert!(result1.is_ok());
        let path1 = result1.unwrap();

        // 同じファイル名で2つ目を作成（連番が付与されるはず）
        let result2 = write_to_file("Second content", filename, Some(dir.path()));
        assert!(result2.is_ok());
        let path2 = result2.unwrap();

        // パスが異なることを確認
        assert_ne!(path1, path2);
        // 連番が付与されていることを確認
        assert!(path2.to_string_lossy().contains("test_export_1.md"));
    }
}
