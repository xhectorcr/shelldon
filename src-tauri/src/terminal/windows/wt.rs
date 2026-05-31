use crate::config::Project;
use crate::terminal::utils::{launch_extra, resolve};
use anyhow::{bail, Result};
use std::process::Command;

pub fn launch_wt(project: &Project) -> Result<()> {
    if project.panes.is_empty() {
        bail!("El proyecto '{}' no tiene panes configurados.", project.name);
    }
    let mut args: Vec<String> = Vec::new();
    let maybe_profile = project.terminal_profile.as_deref();

    // Helper para generar argumentos de un pane
    let wt_pane_args = |pane: &crate::config::Pane, is_split: bool, split_dir: &str, target: Option<&str>| -> Vec<String> {
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
        args.extend(wt_pane_args(&project.panes[0], false, "", None));

        if project.panes.len() > 1 {
            args.extend(wt_pane_args(&project.panes[1], true, "-V", None));
        }

        if project.panes.len() > 2 {
            args.push(";".into());
            args.push("focus-pane".into());
            args.push("-t".into());
            args.push("0".into());
            args.extend(wt_pane_args(&project.panes[2], true, "-H", None));
        }

        if project.panes.len() > 3 {
            args.push(";".into());
            args.push("focus-pane".into());
            args.push("-t".into());
            args.push("1".into());
            args.extend(wt_pane_args(&project.panes[3], true, "-H", None));
        }

        for pane in project.panes.iter().skip(4) {
            args.extend(wt_pane_args(pane, true, "-V", None));
        }
    } else {
        // Layout lineal (columnas por defecto)
        for (i, pane) in project.panes.iter().enumerate() {
            let split_dir = match project.layout.as_str() {
                "cols-2" => "-V",
                _ => "-H",
            };
            args.extend(wt_pane_args(pane, i > 0, split_dir, None));
        }
    }

    for extra in &project.extras { launch_extra(extra)?; }

    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    let mut cmd = Command::new("wt.exe");
    cmd.args(&args);

    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn().map_err(|e| {
        anyhow::anyhow!("No se pudo abrir Windows Terminal: {}. ¿Está instalado?", e)
    })?;
    Ok(())
}
