fn main() {
    nasm_rs::compile_library_args(
        "mss32jumptable",
        &["asm/mss32jumptable.asm"],
        &["-f", "win32", "--prefix", "_"],
    )
    .expect("Failed to assemble NASM sources");

    println!("cargo:rerun-if-changed=asm/mss32jumptable.asm");
    println!("cargo:rustc-link-lib=static=mss32jumptable");
}
