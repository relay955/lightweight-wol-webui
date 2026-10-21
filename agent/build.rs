fn main() {
    println!("cargo:rerun-if-changed=../resources/app-icon.ico");
    let mut res = winres::WindowsResource::new();
    res.set_icon("../resources/app-icon.ico");
    res.compile().expect("Failed to compile agent resources");
}
