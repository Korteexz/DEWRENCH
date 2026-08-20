use std::path::Path;

pub fn is_git_repository(path: &str) -> bool {
    let git_directory = Path::new(path).join(".git");

    git_directory.exists()
}