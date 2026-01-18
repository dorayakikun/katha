use std::io::{self, Stdout};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::backend::CrosstermBackend;

use crate::KathaError;

/// ratatui の Terminal 型エイリアス
pub type RatatuiTerminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

/// ターミナル管理構造体
/// raw mode と alternate screen を管理
pub struct Terminal {
    terminal: RatatuiTerminal,
}

impl Terminal {
    /// 新規作成
    /// raw mode を有効化し、alternate screen に切り替える
    pub fn new() -> Result<Self, KathaError> {
        enable_raw_mode().map_err(|e| KathaError::Terminal(e.to_string()))?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| KathaError::Terminal(e.to_string()))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal =
            RatatuiTerminal::new(backend).map_err(|e| KathaError::Terminal(e.to_string()))?;

        Ok(Self { terminal })
    }

    /// ターミナルへの参照を取得
    pub fn inner(&mut self) -> &mut RatatuiTerminal {
        &mut self.terminal
    }

    /// ターミナルを復元
    pub fn restore(&mut self) -> Result<(), KathaError> {
        disable_raw_mode().map_err(|e| KathaError::Terminal(e.to_string()))?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|e| KathaError::Terminal(e.to_string()))?;
        self.terminal
            .show_cursor()
            .map_err(|e| KathaError::Terminal(e.to_string()))?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // エラーは無視（パニック中の可能性があるため）
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
