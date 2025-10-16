extern crate winres;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/app-icon.ico");
    res.set_resource_file("resources.rc");
    res.compile().expect("Failed to compile resources");
}