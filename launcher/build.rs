fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resource.rc");
    println!("cargo:rerun-if-changed=manifest.xml");

    embed_resource::compile("resource.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
