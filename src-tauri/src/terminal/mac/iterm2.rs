use crate::config::Project;
use crate::terminal::utils::{launch_extra, resolve};
use anyhow::{bail, Result};
use std::process::Command;

pub fn launch_iterm2(project: &Project) -> Result<()> {
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
