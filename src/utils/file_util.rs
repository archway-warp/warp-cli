use std::path::Path;

use crate::error::WarpError;

pub fn replace_in_file<P>(path: P, find: &str, replace: &str) -> Result<(), WarpError>
where
    P: AsRef<Path>,
{
    let mut content = std::fs::read_to_string(&path)?;
    content = content.replace(find, replace);
    std::fs::write(&path, content)?;

    Ok(())
}
