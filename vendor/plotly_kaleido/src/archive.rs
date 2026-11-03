use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Component, Path};

const MAX_UNCOMPRESSED: u64 = 1024 * 1024 * 1024;

fn verify_local_name(reader: &mut fs::File, offset: u64, central_name: &[u8]) -> io::Result<()> {
    reader.seek(SeekFrom::Start(offset))?;
    let mut header = [0_u8; 30];
    reader.read_exact(&mut header)?;
    if &header[..4] != b"PK\x03\x04" {
        return Err(io::Error::other("Invalid local archive header"));
    }
    let length = usize::from(u16::from_le_bytes([header[26], header[27]]));
    if length != central_name.len() {
        return Err(io::Error::other("Inconsistent archive filename"));
    }
    let mut local_name = vec![0_u8; length];
    reader.read_exact(&mut local_name)?;
    if local_name != central_name {
        return Err(io::Error::other("Inconsistent archive filename"));
    }
    Ok(())
}

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
    let source = fs::File::open(archive_path)?;
    let mut header_reader = source.try_clone()?;
    let mut archive = zip::ZipArchive::new(source)?;
    // The random-access API treats central-directory names as authoritative.
    // Explicitly reject dual-header disagreement before creating any payload.
    // by_index_data reads only metadata; subsequent by_index seeks to the entry
    // data again, so the cloned handle's shared cursor cannot affect extraction.
    for index in 0..archive.len() {
        let entry = archive.by_index_data(index)?;
        verify_local_name(&mut header_reader, entry.header_start(), entry.name_raw())?;
    }
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
