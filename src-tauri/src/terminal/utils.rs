use std::process::Command;
use anyhow::Result;

/// Expands Windows environment variables like %SystemRoot% in a string
/// using the native ExpandEnvironmentStringsW API.
#[cfg(target_os = "windows")]
fn expand_env_vars(s: &str) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    extern "system" {
        fn ExpandEnvironmentStringsW(
            lpSrc: *const u16,
            lpDst: *mut u16,
            nSize: u32,
        ) -> u32;
    }

    let wide: Vec<u16> = OsString::from(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // First call: get required buffer size
    let required = unsafe { ExpandEnvironmentStringsW(wide.as_ptr(), std::ptr::null_mut(), 0) };
    if required == 0 {
        return s.to_string();
    }

    let mut buffer: Vec<u16> = vec![0; required as usize];
    let written = unsafe {
        ExpandEnvironmentStringsW(wide.as_ptr(), buffer.as_mut_ptr(), required)
    };
    if written == 0 || written > required {
        return s.to_string();
    }

    // written includes the null terminator
    OsString::from_wide(&buffer[..written as usize - 1])
        .to_string_lossy()
        .into_owned()
}

/// Reads the full PATH from the Windows Registry (Machine + User),
/// expanding environment variables like %SystemRoot%, and merges it
/// with the current process PATH to avoid losing system directories.
#[cfg(target_os = "windows")]
fn get_merged_path() -> String {
    use winreg::enums::*;
    use winreg::RegKey;
    use std::collections::HashSet;

    let machine_path = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
        .and_then(|key| key.get_value::<String, _>("Path"))
        .unwrap_or_default();

    let user_path = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Environment")
        .and_then(|key| key.get_value::<String, _>("Path"))
        .unwrap_or_default();

    // Expand %SystemRoot%, %USERPROFILE%, etc. in registry values
    let machine_expanded = expand_env_vars(&machine_path);
    let user_expanded = expand_env_vars(&user_path);

    // Start with current process PATH (has system32, etc.)
    let current_path = std::env::var("PATH").unwrap_or_default();

    // Deduplicate while preserving order: registry paths first, then current
    let mut seen = HashSet::new();
    let mut parts: Vec<&str> = Vec::new();

    for entry in machine_expanded.split(';')
        .chain(user_expanded.split(';'))
        .chain(current_path.split(';'))
    {
        let trimmed = entry.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_lowercase()) {
            parts.push(trimmed);
        }
    }

    parts.join(";")
}

/// Applies the full reconstructed PATH to a Command so child processes
/// see all system and user tools (php, node, etc.) regardless of how
/// the app was launched.
#[cfg(target_os = "windows")]
pub fn apply_full_path(cmd: &mut Command) {
    let path = get_merged_path();
    if !path.is_empty() {
        cmd.env("PATH", &path);
    }
}

/// Returns a PowerShell snippet that reconstructs PATH at runtime.
/// Use this as a prefix before user commands in PowerShell sessions
/// launched directly (not through wt.exe).
#[cfg(target_os = "windows")]
pub fn ps_path_fix() -> &'static str {
    "$env:PATH = [System.Environment]::GetEnvironmentVariable('PATH','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('PATH','User')"
}

/// Base64-encodes a byte slice (standard alphabet, with padding).
#[cfg(target_os = "windows")]
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Builds a Base64-encoded PowerShell command (UTF-16LE) that:
/// 1. Reconstructs the full PATH from the Windows Registry
/// 2. Loads the user's PowerShell profile (if it exists)
/// 3. Runs the user's command
///
/// Using -EncodedCommand avoids all wt.exe argument parsing issues
/// (semicolons, quotes, etc.) since Base64 contains no special characters.
#[cfg(target_os = "windows")]
pub fn build_ps_encoded_command(user_cmd: &str) -> String {
    let script = format!(
        "$env:PATH = [System.Environment]::GetEnvironmentVariable('PATH','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('PATH','User')\nif (Test-Path $PROFILE) {{ . $PROFILE }}\n{}",
        user_cmd
    );
    // PowerShell -EncodedCommand expects Base64-encoded UTF-16LE
    let utf16le: Vec<u8> = script.encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    base64_encode(&utf16le)
}

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
