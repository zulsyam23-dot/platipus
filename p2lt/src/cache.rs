use std::path::{Path, PathBuf};

/// Where installed packages live inside a project.
pub fn packages_dir(project: &Path) -> PathBuf {
    project.join(".p2lt").join("packages")
}

pub fn package_dir(project: &Path, name: &str) -> PathBuf {
    packages_dir(project).join(name)
}
