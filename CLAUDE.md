# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

katha は Claude Code の会話履歴を閲覧するための TUI（Terminal User Interface）アプリケーションです。Rust で実装されており、ratatui を使用しています。

## Build & Development Commands

```bash
# ビルド
cargo build

# リリースビルド
cargo build --release

# 実行
cargo run

# テスト
cargo test

# 単一テストの実行
cargo test test_name

# 特定モジュールのテスト
cargo test module::test_name

# セッション数のカウント（デバッグ用）
cargo run -- --count-sessions
```

## Architecture

### TEA (The Elm Architecture)

このアプリケーションは TEA パターンを採用しています:

- **Model** (`src/tea/model.rs`): アプリケーション全体の状態を保持
- **Message** (`src/tea/message.rs`): ユーザー操作やイベントを表現
- **Update** (`src/tea/update.rs`): Message に応じて Model を更新

### Module Structure

- `config/`: Claude のパス設定（`~/.claude/` 配下のファイルパス）
- `data/`: データ読み込み
  - `history_reader.rs`: `history.jsonl` のパース
  - `session_reader.rs`: 個別セッションファイルの読み込み
- `domain/`: ドメインモデル（`Session`, `Message`, `HistoryEntry`）
- `export/`: Markdown/JSON 形式へのエクスポート
- `search/`: 検索・フィルタ機能
- `tui/`: ターミナル制御とイベントハンドリング
  - `app.rs`: メインループ
  - `event.rs`: キーボードイベント処理
- `views/`: 各画面のレンダリング
- `widgets/`: 再利用可能な UI コンポーネント

### Data Flow

1. `App::load_sessions()` で `~/.claude/history.jsonl` を読み込み
2. プロジェクトごとにグループ化して `SessionListItem` に変換
3. ユーザーがセッションを選択すると `SessionReader` で詳細を読み込み
4. TEA の update 関数でモデルを更新、views でレンダリング

### View Modes

`ViewMode` enum で画面状態を管理:
- `SessionList`: セッション一覧
- `SessionDetail`: セッション詳細
- `Search`: 検索モード
- `Filter`: フィルタモード
- `Help`: ヘルプ表示
- `Export`: エクスポートダイアログ

## Development Notes

### TUI 実行時の注意

このアプリケーションは TUI（Terminal User Interface）であるため、**インタラクティブなターミナル（TTY）が必要**です。

Claude Code などの非インタラクティブな環境で `cargo run` を実行すると、以下のエラーが発生します:

```
Terminal error: Device not configured (os error 6)
```

**TUI の動作確認は、実際のターミナルで行ってください。**

### デバッグログ

`tracing` クレートを使用したロギングが実装されています。`RUST_LOG` 環境変数でログレベルを制御できます:

```bash
# デバッグログを有効にして実行
RUST_LOG=debug cargo run 2>&1

# より詳細なトレースログ
RUST_LOG=trace cargo run 2>&1
```

ログは stderr に出力されるため、TUI の表示と干渉しません。
