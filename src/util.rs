use std::path::Path;

pub fn lowercase_extension(path: &Path) -> Option<String> {
    let ext = path.extension()?;
    let s = ext.to_str()?;
    Some(s.to_lowercase())
}
