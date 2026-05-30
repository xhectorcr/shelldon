#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cli;
mod commands;
mod config;
mod core;
mod terminal;

use clap::Parser;

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();

    // Si hay argumentos que no son flags de Tauri → modo CLI
    let is_cli = raw_args.len() > 1
        && !raw_args[1].starts_with("--")
        && raw_args[1] != "tauri";

    if is_cli {
        let args = cli::CliArgs::parse();
        if let Err(e) = cli::run(args) {
            eprintln!("❌ ShellDon: {}", e);
            std::process::exit(1);
        }
    } else {
        // Modo GUI — ventana Tauri
        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![
                commands::get_projects,
                commands::save_project,
                commands::delete_project,
                commands::launch_project,
                commands::get_project_by_name,
                commands::open_directory_dialog,
            ])
            .run(tauri::generate_context!())
            .expect("error running ShellDon");
    }
}
