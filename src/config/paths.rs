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
                projects.push(Self::decode_project_path(name));
            }
        }
        Ok(projects)
    }

    /// エンコードされたパスをデコード
    pub fn decode_project_path(encoded: &str) -> String {
        if encoded.starts_with('-') {
            encoded.replacen('-', "/", 1).replace('-', "/")
        } else {
            encoded.replace('-', "/")
        }
    }

    /// パスをエンコード
    pub fn encode_project_path(path: &str) -> String {
        path.replace('/', "-").replace('.', "-").replace('_', "-")
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
            "-Users-test-project"
        );
    }

    #[test]
    fn test_encode_project_path_with_dots() {
        assert_eq!(
            ClaudePaths::encode_project_path("/Users/test/github.com/user/repo"),
            "-Users-test-github-com-user-repo"
        );
    }

    #[test]
    fn test_decode_project_path() {
        assert_eq!(
            ClaudePaths::decode_project_path("-Users-test-project"),
            "/Users/test/project"
        );
    }

    #[test]
    fn test_encode_project_path_with_underscores() {
        assert_eq!(
            ClaudePaths::encode_project_path("/Users/test/ai_agent_status"),
            "-Users-test-ai-agent-status"
        );
    }
}
