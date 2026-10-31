//! Windows only: embed the app icon and version info into `cadcraft.exe`. Elsewhere a no-op.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/tbt360.ico");
    println!("cargo:rerun-if-env-changed=CADCRAFT_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=CADCRAFT_BUILD_SHA");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    if std::path::Path::new("../../assets/app-icon/tbt360.ico").exists() {
        res.set_icon("../../assets/app-icon/tbt360.ico");
    }
    res.set("ProductName", "CAD 2D - TBT 360")
        .set("FileDescription", "CAD 2D - TBT 360 computer-aided design")
        .set("CompanyName", "TBT 360")
        .set("LegalCopyright", "Copyright (c) 2026 TBT 360 contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "tbt360.exe")
        .set("InternalName", "tbt360");
    if let Err(e) = res.compile() {
        if std::env::var_os("CADCRAFT_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=cadcraft.exe built without icon/version resources: {e}");
    }
}
