use std::{
    env::var,
    path::{Path, PathBuf},
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

/// `C:\Program Files (x86)\Windows Kits\10`.
fn read_kits_path() -> PathBuf {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
    let dir: String = hklm
        .open_subkey(key)
        .unwrap()
        .get_value("KitsRoot10")
        .unwrap();

    dir.into()
}

/// Returns the path to the kernel mode libraries. The path may look like this:
/// `C:\Program Files (x86)\Windows Kits\10\lib\10.0.18362.0\km`.
fn find_km_dir(windows_kits_dir: &PathBuf) -> PathBuf {
    let readdir = Path::new(windows_kits_dir).join("lib").read_dir().unwrap();

    let max_libdir = readdir
        .filter_map(|dir| dir.ok())
        .map(|dir| dir.path())
        .filter(|dir| {
            dir.components()
                .last()
                .and_then(|c| c.as_os_str().to_str())
                .map(|c| c.starts_with("10.") && dir.join("km").is_dir())
                .unwrap_or(false)
        })
        .max()
        .unwrap();

    max_libdir.join("km")
}

fn get_target_architecture() -> String {
    let target = var("TARGET").unwrap();
    if target.contains("x86_64") {
        "x64"
    } else if target.contains("i686") {
        "x86"
    } else {
        panic!("Only support x86_64 and i686!")
    }
    .to_string()
}

fn get_kernel_library_dir() -> PathBuf {
    let windows_kits_dir = read_kits_path();
    let km_dir = find_km_dir(&windows_kits_dir);
    let architecture = get_target_architecture();

    km_dir.join(architecture)
}

fn main() {
    let km_dir = get_kernel_library_dir();
    println!(
        "cargo:rustc-link-search=native={}",
        km_dir.to_str().unwrap()
    );
}
