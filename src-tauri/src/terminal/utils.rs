use std::process::Command;
use anyhow::Result;

pub fn get_default_shell() -> String {
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

pub fn launch_extra(extra: &str) -> Result<()> {
    if extra.starts_with("browser:") {
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

pub fn resolve(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return path.replacen('~', &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

pub fn sanitize_session(name: &str) -> String {
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
