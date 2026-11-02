fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=packaging/windows/neocross.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let version = env!("CARGO_PKG_VERSION");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set_manifest_file("packaging/windows/neocross.manifest")
            .set("ProductName", "NeoCross")
            .set("FileDescription", "NeoCross Crosshair Overlay – by T-GOW")
            .set("CompanyName", "T-GOW")
            .set("LegalCopyright", "© 2026 T-GOW")
            .set("ProductVersion", version)
            .set("FileVersion", version);
        if let Err(e) = res.compile() {
            println!("cargo:warning=Could not embed the Windows resources: {e}");
        }
    }
}
