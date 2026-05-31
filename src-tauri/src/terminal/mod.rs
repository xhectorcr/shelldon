pub mod utils;
pub mod windows;
pub mod linux;
pub mod mac;

use crate::config::Project;
use anyhow::{bail, Result};

pub fn launch(project: &Project) -> Result<()> {
    match project.terminal.as_str() {
        "auto" => launch_auto(project),
        "windows-terminal" | "wt" => windows::wt::launch_wt(project),
        "powershell" => windows::powershell::launch_powershell(project),
        "tmux" => linux::tmux::launch_tmux(project),
        "gnome-terminal" => linux::gnome::launch_gnome(project),
        "iterm2" => mac::iterm2::launch_iterm2(project),
        "terminal-app" => mac::terminal_app::launch_terminal_app(project),
        other => bail!("Terminal '{}' no soportada.", other),
    }
}

fn launch_auto(project: &Project) -> Result<()> {
    if cfg!(target_os = "windows") {
        windows::wt::launch_wt(project)
            .or_else(|_| windows::powershell::launch_powershell(project))
    } else if cfg!(target_os = "macos") {
        mac::iterm2::launch_iterm2(project)
            .or_else(|_| mac::terminal_app::launch_terminal_app(project))
    } else {
        linux::tmux::launch_tmux(project)
            .or_else(|_| linux::gnome::launch_gnome(project))
    }
}
