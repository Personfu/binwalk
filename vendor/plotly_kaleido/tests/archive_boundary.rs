#[path = "../src/archive.rs"]
mod archive;

use std::fs;
use std::io::{Cursor, Write};
use tempfile::tempdir;
use zip::write::SimpleFileOptions;

fn fixture(name: &str) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(name, SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"inert fixture").unwrap();
    writer.finish().unwrap().into_inner()
}

#[test]
fn normal_archive_preserves_payload() {
    let root = tempdir().unwrap();
    let zip = root.path().join("fixture.zip");
    fs::write(&zip, fixture("bin/fixture.txt")).unwrap();
    let output = root.path().join("out");
    archive::extract_zip(&output, &zip).unwrap();
    assert_eq!(
        fs::read(output.join("bin/fixture.txt")).unwrap(),
        b"inert fixture"
    );
}

#[test]
fn central_and_local_name_disagreement_is_rejected() {
    let root = tempdir().unwrap();
    let mut bytes = fixture("safe.txt");
    // Harmless equal-length names exercise the advisory's parser disagreement.
    bytes[30..38].copy_from_slice(b"also.txt");
    let zip = root.path().join("fixture.zip");
    fs::write(&zip, bytes).unwrap();
    let output = root.path().join("out");
    assert!(archive::extract_zip(&output, &zip).is_err());
    assert!(!output.join("safe.txt").exists());
    assert!(!output.join("also.txt").exists());
}

#[test]
fn parent_path_is_rejected() {
    let root = tempdir().unwrap();
    let zip = root.path().join("fixture.zip");
    fs::write(&zip, fixture("../outside.txt")).unwrap();
    assert!(archive::extract_zip(&root.path().join("out"), &zip).is_err());
    assert!(!root.path().join("outside.txt").exists());
}

#[test]
fn existing_files_are_never_overwritten() {
    let root = tempdir().unwrap();
    let zip = root.path().join("fixture.zip");
    let output = root.path().join("out");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("safe.txt"), b"original").unwrap();
    fs::write(&zip, fixture("safe.txt")).unwrap();
    assert!(archive::extract_zip(&output, &zip).is_err());
    assert_eq!(fs::read(output.join("safe.txt")).unwrap(), b"original");
}

#[cfg(unix)]
#[test]
fn existing_directory_symlink_cannot_escape_destination() {
    let root = tempdir().unwrap();
    let output = root.path().join("out");
    let outside = root.path().join("outside");
    fs::create_dir(&output).unwrap();
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, output.join("linked")).unwrap();
    let zip = root.path().join("fixture.zip");
    fs::write(&zip, fixture("linked/fixture.txt")).unwrap();
    assert!(archive::extract_zip(&output, &zip).is_err());
    assert!(!outside.join("fixture.txt").exists());
}
