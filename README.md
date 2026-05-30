# ⚡ ShellDon — Developer Workspace Launcher

> Launch your entire dev environment in one click (or one command).

ShellDon is a lightweight desktop launcher that automatically opens your terminal, splits panes, navigates to directories, and runs commands — all from a single JSON config.

## Features

- 🚀 **Instant launch** — One click or `shelldon up <project>`
- 🪟 **Split panes** — Windows Terminal / tmux / iTerm2 support
- 🎨 **Beautiful GUI** — Dark, minimal, fast Tauri + Tailwind UI
- 💻 **CLI-first** — Full command-line interface
- 📁 **JSON config** — Human-readable, git-friendly project configs
- 🔌 **Extras** — Auto-open VSCode, Docker, browser URLs

## Quick Start

### GUI
```bash
npm install
npm run dev       # Tauri dev mode
```

### CLI
```bash
shelldon up erp                    # Launch project "ERP"
shelldon list                      # List all projects
shelldon add ./my-project.json     # Import a project config
shelldon remove erp                # Delete a project
shelldon show erp                  # Show project details
```

## Project Config (JSON)

```json
{
  "name": "ERP",
  "description": "Laravel + Vue full-stack",
  "terminal": "windows-terminal",
  "icon": "🏢",
  "color": "#7c3aed",
  "panes": [
    { "name": "Frontend", "path": "./frontend", "command": "npm run dev" },
    { "name": "Backend",  "path": "./backend",  "command": "php artisan serve" }
  ],
  "extras": ["vscode", "browser:http://localhost:5173"]
}
```

## Supported Terminals

| Terminal | Platform | Split Panes |
|---|---|---|
| Windows Terminal (`wt.exe`) | Windows | ✅ Native |
| PowerShell | Windows | Separate windows |
| CMD | Windows | Separate windows |
| tmux | Linux/macOS | ✅ Native |
| iTerm2 | macOS | ✅ AppleScript |
| Terminator/Tilix | Linux | Separate windows |

## Architecture

```
shelldon/
├── src/                  # Frontend (HTML + Tailwind + Vanilla JS)
│   ├── index.html
│   ├── main.js           # App logic
│   └── api.js            # Tauri IPC bridge (+ browser mock)
└── src-tauri/            # Rust backend
    ├── main.rs           # Entry point (CLI or GUI)
    └── src/
        ├── config.rs     # Project JSON persistence
        ├── terminal.rs   # Terminal adapters
        ├── core.rs       # Orchestration
        ├── commands.rs   # Tauri IPC commands
        └── cli.rs        # CLI (clap)
```

## Data Storage

Projects are saved as JSON files in:
- **Windows:** `%APPDATA%\shelldon\projects\`
- **Linux:** `~/.local/share/shelldon/projects/`
- **macOS:** `~/Library/Application Support/shelldon/projects/`

## Build

```bash
npm install
npm run build     # Tauri production build
```
