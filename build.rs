fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        // The name Windows shows in Task Manager, the taskbar and file properties.
        res.set("FileDescription", "AutoTyper");
        res.set("ProductName", "AutoTyper");
        res.set("InternalName", "AutoTyper");
        res.set("OriginalFilename", "AutoTyper.exe");
        res.compile().expect("failed to embed icon resource");
    }
}
