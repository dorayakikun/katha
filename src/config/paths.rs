use std::path::PathBuf;

use directories::BaseDirs;

use crate::KathaError;

/// Claude Code パス管理
#[derive(Debug, Clone)]
pub struct ClaudePaths {
    pub base_dir: PathBuf,
    pub history_file: PathBuf,
    pub projects_dir: PathBuf,
}

impl ClaudePaths {
    /// デフォルトパスで初期化
    pub fn new() -> Result<Self, KathaError> {
        let base_dirs = BaseDirs::new()
            .ok_or_else(|| KathaError::ConfigError("Cannot find home directory".into()))?;

        let base_dir = base_dirs.home_dir().join(".claude");
        Self::from_base_dir(base_dir)
    }

    /// 指定ディレクトリで初期化
    pub fn from_base_dir(base_dir: PathBuf) -> Result<Self, KathaError> {
        if !base_dir.exists() {
            return Err(KathaError::ConfigError(format!(
                "Claude directory not found: {}",
                base_dir.display()
            )));
        }

        Ok(Self {
            history_file: base_dir.join("history.jsonl"),
            projects_dir: base_dir.join("projects"),
            base_dir,
        })
    }

    pub fn history_exists(&self) -> bool {
        self.history_file.exists()
    }

    pub fn projects_exists(&self) -> bool {
        self.projects_dir.exists()
    }

    /// プロジェクト一覧
    /// Claude Code のディレクトリ構造からプロジェクトパスを読み取る
    /// 注意: Claude Code のエンコードは非可逆のため、'.' や '_' は正確に復元できない
    pub fn list_projects(&self) -> Result<Vec<String>, KathaError> {
        if !self.projects_exists() {
            return Ok(vec![]);
        }

        let mut projects = Vec::new();
        for entry in std::fs::read_dir(&self.projects_dir)? {
            let entry = entry?;
            if entry.path().is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                projects.push(Self::decode_project_path_from_fs(name));
            }
        }
        Ok(projects)
    }

    /// エンコードされたパスをデコード（パーセントエンコーディングを逆変換）
    /// 可逆エンコーディングの逆変換
    pub fn decode_project_path(encoded: &str) -> String {
        encoded
            .replace("%2F", "/")
            .replace("%2E", ".")
            .replace("%5F", "_")
            .replace("%2D", "-")
            .replace("%25", "%")
    }

    /// パスをエンコード（パーセントエンコーディング）
    /// '/', '.', '_', '-' を一意のシーケンスに変換し、可逆性を保証
    pub fn encode_project_path(path: &str) -> String {
        path.replace('%', "%25")
            .replace('/', "%2F")
            .replace('.', "%2E")
            .replace('_', "%5F")
            .replace('-', "%2D")
    }

    /// ファイルシステム用エンコード（Claude Code 互換）
    /// Claude Code が使用するディレクトリ名の形式に変換
    /// 注意: この変換は非可逆（'/', '.', '_' がすべて '-' になる）
    pub fn encode_project_path_for_fs(path: &str) -> String {
        path.replace('/', "-").replace('.', "-").replace('_', "-")
    }

    /// ファイルシステムからのデコード（Claude Code 形式）
    /// Claude Code のディレクトリ名からパスへの変換（非可逆のためベストエフォート）
    /// '-' を '/' に変換するが、元の '.', '_' は復元できない
    pub fn decode_project_path_from_fs(encoded: &str) -> String {
        if encoded.starts_with('-') {
            encoded.replacen('-', "/", 1).replace('-', "/")
        } else {
            encoded.replace('-', "/")
        }
    }
}

/// Codex パス管理
#[derive(Debug, Clone)]
pub struct CodexPaths {
    pub base_dir: PathBuf,
    pub history_file: PathBuf,
    pub sessions_dir: PathBuf,
}

impl CodexPaths {
    /// デフォルトパスで初期化
    pub fn new() -> Result<Self, KathaError> {
        let base_dirs = BaseDirs::new()
            .ok_or_else(|| KathaError::ConfigError("Cannot find home directory".into()))?;

        let base_dir = base_dirs.home_dir().join(".codex");
        Self::from_base_dir(base_dir)
    }

    /// 指定ディレクトリで初期化
    pub fn from_base_dir(base_dir: PathBuf) -> Result<Self, KathaError> {
        if !base_dir.exists() {
            return Err(KathaError::ConfigError(format!(
                "Codex directory not found: {}",
                base_dir.display()
            )));
        }

        Ok(Self {
            history_file: base_dir.join("history.jsonl"),
            sessions_dir: base_dir.join("sessions"),
            base_dir,
        })
    }

    pub fn history_exists(&self) -> bool {
        self.history_file.exists()
    }

    pub fn sessions_exists(&self) -> bool {
        self.sessions_dir.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_project_path() {
        assert_eq!(
            ClaudePaths::encode_project_path("/Users/test/project"),
            "%2FUsers%2Ftest%2Fproject"
        );
    }

    #[test]
    fn test_encode_project_path_with_dots() {
        assert_eq!(
            ClaudePaths::encode_project_path("/Users/test/github.com/user/repo"),
            "%2FUsers%2Ftest%2Fgithub%2Ecom%2Fuser%2Frepo"
        );
    }

    #[test]
    fn test_encode_project_path_with_underscores() {
        assert_eq!(
            ClaudePaths::encode_project_path("/Users/test/ai_agent_status"),
            "%2FUsers%2Ftest%2Fai%5Fagent%5Fstatus"
        );
    }

    #[test]
    fn test_encode_project_path_with_hyphens() {
        assert_eq!(
            ClaudePaths::encode_project_path("/Users/test/my-project"),
            "%2FUsers%2Ftest%2Fmy%2Dproject"
        );
    }

    #[test]
    fn test_encode_project_path_with_percent() {
        assert_eq!(
            ClaudePaths::encode_project_path("/Users/test/100%done"),
            "%2FUsers%2Ftest%2F100%25done"
        );
    }

    #[test]
    fn test_decode_project_path() {
        assert_eq!(
            ClaudePaths::decode_project_path("%2FUsers%2Ftest%2Fproject"),
            "/Users/test/project"
        );
    }

    #[test]
    fn test_decode_project_path_with_dots() {
        assert_eq!(
            ClaudePaths::decode_project_path("%2FUsers%2Ftest%2Fgithub%2Ecom%2Fuser%2Frepo"),
            "/Users/test/github.com/user/repo"
        );
    }

    #[test]
    fn test_decode_project_path_with_underscores() {
        assert_eq!(
            ClaudePaths::decode_project_path("%2FUsers%2Ftest%2Fai%5Fagent%5Fstatus"),
            "/Users/test/ai_agent_status"
        );
    }

    #[test]
    fn test_decode_project_path_with_hyphens() {
        assert_eq!(
            ClaudePaths::decode_project_path("%2FUsers%2Ftest%2Fmy%2Dproject"),
            "/Users/test/my-project"
        );
    }

    #[test]
    fn test_decode_project_path_with_percent() {
        assert_eq!(
            ClaudePaths::decode_project_path("%2FUsers%2Ftest%2F100%25done"),
            "/Users/test/100%done"
        );
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = "/Users/test/my-project_v1.0/src";
        let encoded = ClaudePaths::encode_project_path(original);
        let decoded = ClaudePaths::decode_project_path(&encoded);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_decode_roundtrip_complex() {
        let original = "/Users/test/github.com/user/ai_agent-status_v2.0";
        let encoded = ClaudePaths::encode_project_path(original);
        let decoded = ClaudePaths::decode_project_path(&encoded);
        assert_eq!(decoded, original);
    }

    // ファイルシステム用エンコード（Claude Code 互換）のテスト
    #[test]
    fn test_encode_project_path_for_fs() {
        assert_eq!(
            ClaudePaths::encode_project_path_for_fs("/Users/test/project"),
            "-Users-test-project"
        );
    }

    #[test]
    fn test_encode_project_path_for_fs_with_dots() {
        assert_eq!(
            ClaudePaths::encode_project_path_for_fs("/Users/test/github.com/user/repo"),
            "-Users-test-github-com-user-repo"
        );
    }

    #[test]
    fn test_encode_project_path_for_fs_with_underscores() {
        assert_eq!(
            ClaudePaths::encode_project_path_for_fs("/Users/test/ai_agent_status"),
            "-Users-test-ai-agent-status"
        );
    }

    #[test]
    fn test_decode_project_path_from_fs() {
        assert_eq!(
            ClaudePaths::decode_project_path_from_fs("-Users-test-project"),
            "/Users/test/project"
        );
    }

    #[test]
    fn test_decode_project_path_from_fs_without_leading_slash() {
        assert_eq!(
            ClaudePaths::decode_project_path_from_fs("Users-test-project"),
            "Users/test/project"
        );
    }
}
