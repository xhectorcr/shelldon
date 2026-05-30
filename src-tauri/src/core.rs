use crate::config::Project;
use crate::terminal;
use anyhow::Result;

pub fn launch_project(project: &Project) -> Result<()> {
    terminal::launch(project)
}
