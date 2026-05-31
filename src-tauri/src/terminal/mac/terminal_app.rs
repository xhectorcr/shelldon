use crate::config::Project;
use crate::terminal::utils::{launch_extra, resolve};
use anyhow::Result;
use std::process::Command;

pub fn launch_terminal_app(project: &Project) -> Result<()> {
    for pane in &project.panes {
        let shell_cmd = format!("cd '{}' && {}", resolve(&pane.path), pane.command);
        // Use AppleScript to tell Terminal to do script
        let script = format!(
            "tell application \"Terminal\"\n  do script \"{}\"\n  activate\nend tell",
            shell_cmd
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| anyhow::anyhow!("Terminal.app: {}", e))?;
    }
    
    for extra in &project.extras {
        launch_extra(extra)?;
    }
    Ok(())
}
