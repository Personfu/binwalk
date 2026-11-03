extern crate directories;
extern crate zip;

use std::env;
use std::io::Result;
use std::path::PathBuf;
use std::process::Command;

use directories::ProjectDirs;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const KALEIDO_URL: &str =
    "https://github.com/plotly/Kaleido/releases/download/v0.2.1/kaleido_linux_x64.zip";

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
const KALEIDO_URL: &str =
    "https://github.com/plotly/Kaleido/releases/download/v0.2.1/kaleido_linux_arm64.zip";

#[cfg(target_os = "windows")]
const KALEIDO_URL: &str =
    "https://github.com/plotly/Kaleido/releases/download/v0.2.1/kaleido_win_x64.zip";

#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
const KALEIDO_URL: &str =
    "https://github.com/plotly/Kaleido/releases/download/v0.2.1/kaleido_mac_x64.zip";

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
const KALEIDO_URL: &str =
    "https://github.com/plotly/Kaleido/releases/download/v0.2.1/kaleido_mac_arm64.zip";

#[cfg(any(target_os = "linux", target_os = "macos"))]
const KALEIDO_BIN: &str = "kaleido";

#[cfg(target_os = "windows")]
const KALEIDO_BIN: &str = "kaleido.exe";

#[path = "src/archive.rs"]
mod archive;

fn main() -> Result<()> {
    if cfg!(feature = "download") {
        let project_dirs = ProjectDirs::from("org", "plotly", "kaleido")
            .expect("Could not create Kaleido config directory path.");
        let dst: PathBuf = project_dirs.config_dir().into();

        let kaleido_binary = dst.join("bin").join(KALEIDO_BIN);

        println!("cargo:rerun-if-changed=src/lib.rs");
        println!(
            "cargo::rerun-if-changed={}",
            kaleido_binary.to_string_lossy()
        );

        println!(
            "cargo:rustc-env=KALEIDO_COMPILE_TIME_DLD_PATH={}",
            dst.to_string_lossy()
        );

        if kaleido_binary.exists() {
            return Ok(());
        }

        let msg = format!(
            "Downloaded Plotly Kaleido from {KALEIDO_URL} to '{}'",
            dst.to_string_lossy()
        );
        println!("cargo::warning={msg}");

        let p = PathBuf::from(env::var("OUT_DIR").unwrap());
        let kaleido_zip_file = p.join("kaleido.zip");

        // Use the platform HTTPS client; never install an unpinned executable
        // from inside a Cargo build script.
        let status = Command::new("curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                "--tlsv1.2",
                "--connect-timeout",
                "20",
                "--max-time",
                "240",
                "--output",
            ])
            .arg(&kaleido_zip_file)
            .arg(KALEIDO_URL)
            .status()?;
        if !status.success() {
            return Err(std::io::Error::other("Kaleido download failed"));
        }
        archive::extract_zip(&dst, &kaleido_zip_file)?;
    } else {
        let msg = "'download' feature disabled. Please install Kaleido manually and make the environment variable 'KALEIDO_PATH' point to it.".to_string();
        println!("cargo::warning={msg}");
    }
    Ok(())
}
