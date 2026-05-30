use anyhow::{Context, Result};
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pane {
    pub name:    String,
    pub path:    String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name:        String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_terminal")]
    pub terminal:    String,
    #[serde(default)]
    pub color:       String,
    #[serde(default = "default_icon")]
    pub icon:        String,
    #[serde(default = "default_layout")]
    pub layout:      String,
    /// Perfil de Windows Terminal (ej: "PowerShell", "Ubuntu", "Git Bash")
    #[serde(default)]
    pub terminal_profile: Option<String>,
    /// Shell a usar (ej: "pwsh.exe", "bash", "zsh"). None = usar el del sistema.
    #[serde(default)]
    pub shell: Option<String>,
    /// Si debe cargar el perfil/dotfiles del usuario automáticamente.
    #[serde(default = "default_true")]
    pub load_profile: bool,
    pub panes:       Vec<Pane>,
    #[serde(default)]
    pub extras:      Vec<String>,
}

fn default_terminal() -> String {
    if cfg!(target_os = "windows") { "windows-terminal".into() }
    else if cfg!(target_os = "macos") { "iterm2".into() }
    else { "tmux".into() }
}
fn default_icon() -> String { "🚀".into() }
fn default_layout() -> String { "cols-2".into() }
fn default_true() -> bool { true }

pub fn data_directory() -> Result<PathBuf> {
    let base = data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir  = base.join("shelldon").join("projects");
    fs::create_dir_all(&dir)
        .with_context(|| format!("No se pudo crear el directorio: {}", dir.display()))?;
    Ok(dir)
}

pub fn load_all_projects() -> Result<Vec<Project>> {
    let dir = data_directory()?;
    let mut projects = Vec::new();
    for entry in fs::read_dir(&dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            match serde_json::from_str::<Project>(&content) {
                Ok(p)  => projects.push(p),
                Err(e) => eprintln!("⚠️  Config inválida {}: {}", path.display(), e),
            }
        }
    }
    projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(projects)
}

pub fn load_project(name: &str) -> Result<Option<Project>> {
    Ok(load_all_projects()?.into_iter()
        .find(|p| p.name.to_lowercase() == name.to_lowercase()))
}

pub fn save_project(project: &Project) -> Result<()> {
    let path = data_directory()?.join(sanitize(&project.name) + ".json");
    fs::write(&path, serde_json::to_string_pretty(project)?)
        .with_context(|| format!("No se pudo guardar: {}", path.display()))
}

pub fn delete_project(name: &str) -> Result<bool> {
    let path = data_directory()?.join(sanitize(name) + ".json");
    if path.exists() { fs::remove_file(&path)?; Ok(true) } else { Ok(false) }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}
