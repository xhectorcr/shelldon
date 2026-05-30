use crate::config::{self, Project};
use crate::core;
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "shelldon", about = "ShellDon — Developer Workspace Launcher", version = "0.1.0")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Lanza un proyecto por nombre
    #[command(alias = "u")]
    Up { name: String },
    /// Lista todos los proyectos guardados
    #[command(alias = "ls")]
    List,
    /// Importa un proyecto desde un archivo JSON
    Add { file: String },
    /// Elimina un proyecto por nombre
    #[command(alias = "rm")]
    Remove { name: String },
    /// Muestra los detalles de un proyecto
    #[command(alias = "info")]
    Show { name: String },
}

pub fn run(args: CliArgs) -> Result<()> {
    match args.command {
        Commands::Up     { name }  => cmd_up(&name),
        Commands::List             => cmd_list(),
        Commands::Add    { file }  => cmd_add(&file),
        Commands::Remove { name }  => cmd_remove(&name),
        Commands::Show   { name }  => cmd_show(&name),
    }
}

fn cmd_up(name: &str) -> Result<()> {
    match config::load_project(name)? {
        Some(p) => { println!("🚀 Lanzando \"{}\"...", p.name); core::launch_project(&p)?; println!("✅ Entorno lanzado."); Ok(()) }
        None    => bail!("Proyecto '{}' no encontrado. Usa 'shelldon list' para ver los disponibles.", name),
    }
}

fn cmd_list() -> Result<()> {
    let projects = config::load_all_projects()?;
    if projects.is_empty() { println!("Sin proyectos. Usa 'shelldon add <archivo.json>' para agregar uno."); return Ok(()); }
    println!("📋 Proyectos configurados:\n");
    for p in &projects {
        let n = p.panes.len();
        println!("  {} {:<20} — {} pane{} — {}", p.icon, p.name, n, if n == 1 { "" } else { "s" }, p.description);
    }
    println!("\nUsa 'shelldon up <nombre>' para lanzar.");
    Ok(())
}

fn cmd_add(file: &str) -> Result<()> {
    let content = std::fs::read_to_string(file).with_context(|| format!("No se puede leer: {}", file))?;
    let project: Project = serde_json::from_str(&content).with_context(|| "JSON inválido")?;
    let name = project.name.clone();
    config::save_project(&project)?;
    println!("✅ Proyecto '{}' agregado.", name);
    Ok(())
}

fn cmd_remove(name: &str) -> Result<()> {
    if config::delete_project(name)? { println!("🗑️  Proyecto '{}' eliminado.", name); }
    else { println!("⚠️  Proyecto '{}' no encontrado.", name); }
    Ok(())
}

fn cmd_show(name: &str) -> Result<()> {
    match config::load_project(name)? {
        Some(p) => {
            println!("{} {}", p.icon, p.name);
            if !p.description.is_empty() { println!("   Descripción: {}", p.description); }
            println!("   Terminal: {}", p.terminal);
            for pane in &p.panes { println!("   • {} → {} → `{}`", pane.name, pane.path, pane.command); }
            if !p.extras.is_empty() { println!("   Extras: {}", p.extras.join(", ")); }
            Ok(())
        }
        None => bail!("Proyecto '{}' no encontrado.", name),
    }
}
