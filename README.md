# katha /ˈkɑːtʰɑː/

A Terminal User Interface (TUI) for browsing Claude Code conversation history.

## Features

- Browse session history grouped by project
- Hierarchical project tree view with expand/collapse functionality
- Full-text search across projects and conversations
- Filter by date range (Today, Last 7 days, Last 30 days) and project name
- Export sessions to Markdown or JSON format
- Two-pane layout with session list and preview

## Installation

### From GitHub

```bash
cargo install --git https://github.com/dorayakikun/katha
```

### From source

```bash
git clone https://github.com/dorayakikun/katha
cd katha
cargo install --path .
```

## Usage

```bash
katha
```

## Key Bindings

### Navigation
| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `l` / `→` | Expand project |
| `h` / `←` | Collapse project |
| `E` | Expand all projects |
| `C` | Collapse all projects |
| `Enter` | View session details |
| `Esc` | Back / Clear filters |
| `q` | Quit |

### Search & Filter
| Key | Action |
|-----|--------|
| `/` | Search mode |
| `f` | Filter panel |
| `Tab` | Switch filter fields |
| `c` | Clear filters (in filter mode) |

### Export & Help
| Key | Action |
|-----|--------|
| `e` | Export dialog |
| `?` | Help |

### Detail Actions
| Key | Action |
|-----|--------|
| `y` | Copy selected message |
| `Y` | Copy selected message with meta |

## Requirements

- Rust 1.70+
- Claude Code history data (`~/.claude/`)
- Codex history data (`~/.codex/`) (optional)
