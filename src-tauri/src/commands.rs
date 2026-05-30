use crate::config::{self, Project};
use crate::core;
use tauri::command;

#[command]
pub fn get_projects() -> Result<Vec<Project>, String> {
    config::load_all_projects().map_err(|e| e.to_string())
}

#[command]
pub fn get_project_by_name(name: String) -> Result<Option<Project>, String> {
    config::load_project(&name).map_err(|e| e.to_string())
}

#[command]
pub fn save_project(project: Project) -> Result<(), String> {
    config::save_project(&project).map_err(|e| e.to_string())
}

#[command]
pub fn delete_project(name: String) -> Result<bool, String> {
    config::delete_project(&name).map_err(|e| e.to_string())
}

#[command]
pub fn launch_project(name: String) -> Result<(), String> {
    match config::load_project(&name).map_err(|e| e.to_string())? {
        Some(p) => core::launch_project(&p).map_err(|e| e.to_string()),
        None    => Err(format!("Proyecto '{}' no encontrado.", name)),
    }
}

#[command]
pub async fn open_directory_dialog(_app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;
    Ok(FileDialogBuilder::new().pick_folder().map(|p| p.to_string_lossy().to_string()))
}
