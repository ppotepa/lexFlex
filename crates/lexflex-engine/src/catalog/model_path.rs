pub use super::model_loader::ModelLoadError;
use std::path::{Component, Path, PathBuf};

pub fn safe_package_path(root: &Path, relative: &str) -> Result<PathBuf, ModelLoadError> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ModelLoadError::UnsafePath {
            relative: relative.to_path_buf(),
        });
    }
    Ok(root.join(relative))
}
