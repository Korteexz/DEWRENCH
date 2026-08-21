use std::path::Path;
use std::process::Command;

pub fn run(
    path: &Path,
    args: &[&str],
) -> Result<String, String> {
    Ok(
        run_raw(path, args)?
            .trim()
            .to_string()
    )
}

pub fn run_raw(
    path: &Path,
    args: &[&str],
) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|error| {
            format!("Não foi possível executar Git: {error}")
        })?;

    if !output.status.success() {
        return Err(
            String::from_utf8_lossy(&output.stderr)
                .trim()
                .to_string()
        );
    }

    Ok(
        String::from_utf8_lossy(&output.stdout)
            .into_owned()
    )
}