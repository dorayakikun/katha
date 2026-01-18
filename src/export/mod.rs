pub mod json;
pub mod markdown;
pub mod writer;

use crate::domain::Session;

/// エクスポート形式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    /// Markdown 形式
    #[default]
    Markdown,
    /// JSON 形式
    Json,
}

impl ExportFormat {
    /// ファイル拡張子を取得
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "md",
            ExportFormat::Json => "json",
        }
    }

    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "Markdown",
            ExportFormat::Json => "JSON",
        }
    }

    /// 次の形式に切り替え
    pub fn next(&self) -> Self {
        match self {
            ExportFormat::Markdown => ExportFormat::Json,
            ExportFormat::Json => ExportFormat::Markdown,
        }
    }
}

/// エクスポーターのトレイト
pub trait Exporter {
    /// セッションを文字列に変換
    fn export(&self, session: &Session) -> String;

    /// ファイル拡張子
    fn file_extension(&self) -> &'static str;
}

pub use json::JsonExporter;
pub use markdown::MarkdownExporter;
pub use writer::{generate_filename, write_to_file};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Markdown.extension(), "md");
        assert_eq!(ExportFormat::Json.extension(), "json");
    }

    #[test]
    fn test_export_format_display_name() {
        assert_eq!(ExportFormat::Markdown.display_name(), "Markdown");
        assert_eq!(ExportFormat::Json.display_name(), "JSON");
    }

    #[test]
    fn test_export_format_next() {
        assert_eq!(ExportFormat::Markdown.next(), ExportFormat::Json);
        assert_eq!(ExportFormat::Json.next(), ExportFormat::Markdown);
    }
}
