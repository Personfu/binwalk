use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path};

const MAX_UNCOMPRESSED: u64 = 1024 * 1024 * 1024;

fn safe_directory(root: &Path, relative: &Path) -> io::Result<()> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(io::Error::other("Unsafe archive directory"));
        };
        current.push(name);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => return Err(io::Error::other("Unsafe existing archive directory")),
            Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&current)?,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

pub fn extract_zip(destination: &Path, archive_path: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    if fs::symlink_metadata(destination)?.file_type().is_symlink() {
        return Err(io::Error::other("Archive destination is a symlink"));
    }
    let mut archive = zip::ZipArchive::new(fs::File::open(archive_path)?)?;
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| io::Error::other("Unsafe or inconsistent archive filename"))?;
        if entry.is_symlink() {
            return Err(io::Error::other("Archive symlinks are not supported"));
        }
        total = total
            .checked_add(entry.size())
            .filter(|size| *size <= MAX_UNCOMPRESSED)
            .ok_or_else(|| io::Error::other("Archive exceeds extraction budget"))?;
        let output = destination.join(&relative);
        if entry.is_dir() {
            safe_directory(destination, &relative)?;
        } else {
            if let Some(parent) = relative.parent() {
                safe_directory(destination, parent)?;
            }
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&output)?;
            let copied = io::copy(&mut entry.by_ref().take(MAX_UNCOMPRESSED + 1), &mut file)?;
            if copied != entry.size() {
                return Err(io::Error::other("Archive entry size mismatch"));
            }
            #[cfg(unix)]
            if let Some(mode) = entry.unix_mode() {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&output, fs::Permissions::from_mode(mode & 0o777))?;
            }
        }
    }
    fs::remove_file(archive_path)
}
