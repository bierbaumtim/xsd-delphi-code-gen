#[allow(clippy::all)]
extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let res = winres::WindowsResource::new();
        // res.set_icon("test.ico");
        res.compile().unwrap();
    }
}
