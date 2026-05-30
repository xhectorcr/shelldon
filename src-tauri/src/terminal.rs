use crate::config::Project;
use anyhow::{bail, Result};
use std::process::Command;

pub fn launch(project: &Project) -> Result<()> {
    match project.terminal.as_str() {
        "windows-terminal" | "wt" => launch_wt(project),
        "powershell" => launch_powershell(project),
        "pwsh" => launch_pwsh(project), // PowerShell 7+ con perfil
        "cmd" => launch_cmd(project),
        "tmux" => launch_tmux(project),
        "iterm2" => launch_iterm2(project),
        "terminator" | "tilix" => launch_tabs(project, &project.terminal.clone()),
        other => bail!("Terminal '{}' no soportada.", other),
    }
}

// ── Windows Terminal (con perfil personalizado) ─────────────────────────────
fn launch_wt(project: &Project) -> Result<()> {
    if project.panes.is_empty() {
        bail!("El proyecto '{}' no tiene panes configurados.", project.name);
    }
    let mut args: Vec<String> = Vec::new();
    let maybe_profile = project.terminal_profile.as_deref();

    // Helper para generar argumentos de un pane
    let mut wt_pane_args = |pane: &crate::config::Pane, is_split: bool, split_dir: &str, target: Option<&str>| -> Vec<String> {
        let dir = resolve(&pane.path);
        let cmd = pane.command.trim();
        let mut v: Vec<String> = Vec::new();
        
        if is_split {
            v.push(";".into());
            v.push("split-pane".into());
            if let Some(t) = target {
                v.push("-t".into());
                v.push(t.into());
            }
            v.push(split_dir.into());
        } else {
            v.push("new-tab".into());
        }

        v.extend(["--title".into(), pane.name.clone(), "--startingDirectory".into(), dir]);
        if let Some(profile) = maybe_profile {
            v.extend(["--profile".into(), profile.into()]);
        }

        v.push("--".into());
        
        // Detección de Shell Inteligente
        if cmd.to_lowercase().contains("pwsh") {
            v.extend(["pwsh.exe".into(), "-NoExit".into(), "-Command".into(), cmd.into()]);
        } else if cmd.to_lowercase().contains("bash") {
            v.extend(["bash.exe".into(), "-c".into(), format!("{}; exec bash", cmd)]);
        } else {
            v.extend(["powershell.exe".into(), "-NoExit".into(), "-Command".into(), cmd.into()]);
        }
        v
    };

    if project.layout == "grid-2x2" {
        for (i, pane) in project.panes.iter().enumerate() {
            let (is_split, split_dir, target) = match i {
                0 => (false, "", None),
                1 => (true, "-V", None),      // Split vertical de P0 -> [P0 | P1]
                2 => (true, "-H", Some("0")), // Split horizontal de P0 -> [P0/P2 | P1]
                3 => (true, "-H", Some("1")), // Split horizontal de P1 -> [P0/P2 | P1/P3]
                _ => (true, "-V", None),      // El resto sigue dividiendo el último
            };
            args.extend(wt_pane_args(pane, is_split, split_dir, target));
        }
    } else {
        // Layout lineal (columnas por defecto)
        for (i, pane) in project.panes.iter().enumerate() {
            let split_dir = match project.layout.as_str() {
                "cols-2" | "cols-3" => "-V",
                _ => "-H",
            };
            args.extend(wt_pane_args(pane, i > 0, split_dir, None));
        }
    }

    for extra in &project.extras { launch_extra(extra)?; }

    Command::new("wt.exe").args(&args).spawn().map_err(|e| {
        anyhow::anyhow!("No se pudo abrir Windows Terminal: {}. ¿Está instalado?", e)
    })?;
    Ok(())
}

// ── PowerShell (Windows PowerShell, sin perfil automático) ───────────────────
fn launch_powershell(project: &Project) -> Result<()> {
    for pane in &project.panes {
        // Forzar carga del perfil de PowerShell
        let script = format!(
            "& {{ $profile_path = if (Test-Path $PROFILE.CurrentUserCurrentHost) {{ $PROFILE.CurrentUserCurrentHost }} else {{ $PROFILE.AllUsersCurrentHost }}; if (Test-Path $profile_path) {{ . $profile_path }}; Set-Location '{}'; {} }}",
            resolve(&pane.path).replace('\\', "\\\\").replace('\'', "''"),
            pane.command
        );

        Command::new("powershell.exe")
            .args(["-NoExit", "-Command", &script])
            .spawn()
            .map_err(|e| anyhow::anyhow!("PowerShell: {}", e))?;
    }
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}

// ── PowerShell Core (pwsh) con perfil automático ────────────────────────────
fn launch_pwsh(project: &Project) -> Result<()> {
    for pane in &project.panes {
        // pwsh carga automáticamente tu perfil, solo necesitamos el comando
        let script = format!("Set-Location '{}'; {}", resolve(&pane.path), pane.command);

        Command::new("pwsh.exe")
            .args(["-NoExit", "-Command", &script])
            .spawn()
            .map_err(|e| anyhow::anyhow!("PowerShell 7: {}", e))?;
    }
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}

// ── CMD ──────────────────────────────────────────────────────────────────────
fn launch_cmd(project: &Project) -> Result<()> {
    for pane in &project.panes {
        // CMD no tiene perfil personalizado, pero podemos cargar AutoRun si existe
        let script = format!(
            "cd /d \"{}\" && if defined AUTORUN (%%AUTORUN%%) && {}",
            resolve(&pane.path),
            pane.command
        );

        Command::new("cmd.exe")
            .args(["/K", &script])
            .spawn()
            .map_err(|e| anyhow::anyhow!("CMD: {}", e))?;
    }
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}

// ── tmux (con entorno completo) ─────────────────────────────────────────────
fn launch_tmux(project: &Project) -> Result<()> {
    if project.panes.is_empty() {
        bail!("Sin panes.");
    }
    let session = sanitize_session(&project.name);
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .output();

    let first = &project.panes[0];
    let default_shell = get_default_shell();

    // Crear sesión con tu shell por defecto
    Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s",
            &session,
            "-c",
            &resolve(&first.path),
            &default_shell,
        ])
        .spawn()?
        .wait()?;

    // Enviar comando con sourcing de dotfiles
    let cmd = format!(
        "source ~/.bashrc 2>/dev/null || source ~/.zshrc 2>/dev/null; {}",
        first.command
    );
    Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}:0.0", session),
            &cmd,
            "Enter",
        ])
        .spawn()?
        .wait()?;

    for (i, pane) in project.panes.iter().enumerate().skip(1) {
        let (split_arg, target) = match project.layout.as_str() {
            "grid-2x2" => {
                if i == 1 {
                    ("-h", format!("{}:0.0", session))
                } else if i == 2 {
                    ("-v", format!("{}:0.0", session))
                } else if i == 3 {
                    ("-v", format!("{}:0.1", session))
                } else {
                    ("-v", format!("{}:0", session))
                }
            }
            "cols-2" | "cols-3" => ("-h", format!("{}:0", session)),
            "rows-2" | "rows-3" => ("-v", format!("{}:0", session)),
            "focus-right" => {
                if i == 1 {
                    ("-h", format!("{}:0", session))
                } else {
                    ("-v", format!("{}:0.1", session))
                }
            }
            "focus-left" => {
                if i == 1 {
                    ("-h", format!("{}:0", session))
                } else {
                    ("-v", format!("{}:0.0", session))
                }
            }
            _ => ("-v", format!("{}:0", session)),
        };

        Command::new("tmux")
            .args([
                "split-window",
                split_arg,
                "-t",
                &target,
                "-c",
                &resolve(&pane.path),
                &default_shell,
            ])
            .spawn()?
            .wait()?;

        let cmd = format!(
            "source ~/.bashrc 2>/dev/null || source ~/.zshrc 2>/dev/null; {}",
            pane.command
        );
        Command::new("tmux")
            .args([
                "send-keys",
                "-t",
                &format!("{}:0.{}", session, i),
                &cmd,
                "Enter",
            ])
            .spawn()?
            .wait()?;
    }

    Command::new("tmux")
        .args(["attach-session", "-t", &session])
        .spawn()?;
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}

// ── iTerm2 (macOS con tu perfil de shell) ───────────────────────────────────
fn launch_iterm2(project: &Project) -> Result<()> {
    if project.panes.is_empty() {
        bail!("Sin panes.");
    }
    let mut script = String::from("tell application \"iTerm2\"\n  activate\n  set w to (create window with default profile)\n  tell current session of w\n");

    let first = &project.panes[0];
    let shell_cmd = format!(
        "cd '{}' && source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null && {}",
        resolve(&first.path),
        first.command
    );
    script.push_str(&format!("    write text \"{}\"\n", shell_cmd));

    let split_cmd = if project.layout == "vertical" || project.layout == "cols-2" {
        "split vertically"
    } else if project.layout == "grid-2x2" {
        "split vertically"
    } else {
        "split horizontally"
    };

    for pane in project.panes.iter().skip(1) {
        script.push_str(&format!(
            "  end tell\n  tell w\n    set p to ({} with default profile)\n  end tell\n  tell p\n",
            split_cmd
        ));
        let shell_cmd = format!(
            "cd '{}' && source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null && {}",
            resolve(&pane.path),
            pane.command
        );
        script.push_str(&format!("    write text \"{}\"\n", shell_cmd));
    }
    script.push_str("  end tell\nend tell\n");

    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| anyhow::anyhow!("iTerm2 AppleScript: {}", e))?;
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}

// ── Terminator / Tilix (con bash interactivo) ───────────────────────────────
fn launch_tabs(project: &Project, app: &str) -> Result<()> {
    for pane in &project.panes {
        // Usar bash interactivo para cargar .bashrc
        let cmd = format!(
            "bash -ic 'cd \"{}\" && {}; exec bash'",
            resolve(&pane.path),
            pane.command
        );

        Command::new(app)
            .args(["--working-directory", &resolve(&pane.path), "-e", &cmd])
            .spawn()
            .map_err(|e| anyhow::anyhow!("{}: {}", app, e))?;
    }
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}

// ── Funciones auxiliares ────────────────────────────────────────────────────
fn find_first_pane_index(node: &serde_json::Value) -> Option<usize> {
    if node["type"] == "pane" {
        Some(node["index"].as_u64().unwrap_or(0) as usize)
    } else if node["type"] == "split" {
        find_first_pane_index(&node["left"]).or_else(|| find_first_pane_index(&node["right"]))
    } else {
        None
    }
}

fn get_shell_command(command: &str, profile: &str) -> String {
    if profile.contains("pwsh") {
        format!("pwsh.exe -NoExit -Command {}", command)
    } else if profile.contains("powershell") {
        format!("powershell.exe -NoExit -Command {}", command)
    } else if profile.contains("bash") {
        format!("bash -ic '{} ; exec bash'", command)
    } else if profile.contains("zsh") {
        format!("zsh -ic '{} ; exec zsh'", command)
    } else {
        command.to_string()
    }
}

fn get_default_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if cfg!(windows) {
                "pwsh.exe".to_string()
            } else {
                "/bin/bash".to_string()
            }
        })
}

fn launch_extra(extra: &str) -> Result<()> {
    if extra == "vscode" || extra == "code" {
        Command::new("code")
            .arg(".")
            .spawn()
            .map_err(|e| anyhow::anyhow!("VSCode: {}. ¿Está 'code' en el PATH?", e))?;
    } else if extra.starts_with("browser:") {
        open_url(extra.trim_start_matches("browser:"))?;
    } else if extra.starts_with("http://") || extra.starts_with("https://") {
        open_url(extra)?;
    } else {
        Command::new(extra)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Extra '{}': {}", extra, e))?;
    }
    Ok(())
}

fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    Command::new("cmd").args(["/C", "start", "", url]).spawn()?;
    #[cfg(target_os = "macos")]
    Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

fn resolve(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return path.replacen('~', &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

fn sanitize_session(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
