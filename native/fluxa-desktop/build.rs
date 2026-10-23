fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../../packaging/icons/fluxa.ico");
        resource.set("ProductName", "Fluxa");
        resource.set("FileDescription", "Fluxa");
        resource.compile().expect("embed Windows icon");
    }
}
