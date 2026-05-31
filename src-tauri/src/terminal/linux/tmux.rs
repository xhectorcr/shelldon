use crate::config::Project;
use crate::terminal::utils::{launch_extra, resolve, sanitize_session, get_default_shell};
use anyhow::{bail, Result};
use std::process::Command;

pub fn launch_tmux(project: &Project) -> Result<()> {
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
