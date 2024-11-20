use git2::Config as GitConfig;

use std::path::PathBuf;
use std::process::Command;

use crate::errors::Error;

fn get_editor() -> Result<String, Error> {
    if let Ok(config) = GitConfig::open_default() {
        if let Ok(editor) = config.get_string("core.editor") {
            return Ok(editor);
        }
    }

    Ok(std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vim".to_string()))
}

pub fn open_editor(filepath: &PathBuf) -> Result<(), Error> {
    let editor_cmd = get_editor().unwrap();
    let parts: Vec<String> = shell_words::split(&editor_cmd)
        .map_err(|e| Error::Other(format!("Failed to parse editor command: {}", e).into()))?;

    if parts.is_empty() {
        return Err(Error::Other("Empty editor command".into()));
    }

    let mut cmd = Command::new(&parts[0]);
    cmd.args(&parts[1..]).arg(filepath);

    cmd.status().map_err(|e| {
        Error::Other(format!("Failed to open editor ({:?}): {}", editor_cmd, e).into())
    })?;

    Ok(())
}