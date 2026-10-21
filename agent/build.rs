fn main() {
    println!("cargo:rerun-if-changed=../resources/app-icon.ico");
    println!("cargo:rerun-if-changed=../resources/tray-icon.ico");
    let mut res = winres::WindowsResource::new();
    res.set_icon("../resources/app-icon.ico");
    res.append_rc_content("exe-icon ICON \"../resources/tray-icon.ico\"");
    res.compile().expect("Failed to compile agent resources");
}
