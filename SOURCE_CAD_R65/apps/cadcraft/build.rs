//! Windows only: embed the app icon and version info into `cadcraft.exe`. Elsewhere a no-op.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/tbt360.ico");
    println!("cargo:rerun-if-env-changed=CADCRAFT_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=CADCRAFT_BUILD_SHA");
    println!("cargo:rerun-if-env-changed=CAD360_RELEASE_VERSION");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let strict = std::env::var_os("CADCRAFT_REQUIRE_WINRES").is_some();
    let release_version = std::env::var("CAD360_RELEASE_VERSION").unwrap_or_else(|_| "0.69.0".to_string());
    let parsed: Result<Vec<u64>, _> = release_version.split('.').map(str::parse::<u64>).collect();
    let parts = match parsed {
        Ok(parts) if (3..=4).contains(&parts.len()) => parts,
        _ if strict => {
            eprintln!("invalid CAD360_RELEASE_VERSION for Windows resources: {release_version}");
            std::process::exit(1);
        }
        _ => vec![0, 67, 0],
    };
    let major = parts[0].min(u16::MAX as u64);
    let minor = parts[1].min(u16::MAX as u64);
    let patch = parts[2].min(u16::MAX as u64);
    let build = parts.get(3).copied().unwrap_or(0).min(u16::MAX as u64);
    let packed_version = (major << 48) | (minor << 32) | (patch << 16) | build;
    let icon = std::path::Path::new("../../assets/app-icon/tbt360.ico");
    if strict && !icon.is_file() {
        eprintln!("required Windows icon is missing: {}", icon.display());
        std::process::exit(1);
    }
    let mut res = winresource::WindowsResource::new();
    if icon.is_file() {
        res.set_icon(icon.to_string_lossy().as_ref());
    }
    res.set("ProductName", "CAD 360 Việt Nam - TBT")
        .set("FileDescription", "CAD 360 Việt Nam - TBT - CAD 2D and quantity takeoff")
        .set("FileVersion", &release_version)
        .set("ProductVersion", &release_version)
        .set("CompanyName", "Công ty CP CN TBT Việt Nam")
        .set("LegalCopyright", "Copyright (c) 2026 TBT Việt Nam. MIT OR Apache-2.0.")
        .set("OriginalFilename", "CAD360.exe")
        .set("InternalName", "CAD360")
        .set_version_info(winresource::VersionInfo::FILEVERSION, packed_version)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, packed_version);
    if let Err(e) = res.compile() {
        if strict {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=cadcraft.exe built without icon/version resources: {e}");
    }
}
