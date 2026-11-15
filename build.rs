fn main() {
    embed_manifest::embed_manifest(embed_manifest::new_manifest("pixel2resolve"))
        .expect("could not embed the Windows manifest");
    println!("cargo:rerun-if-changed=build.rs");
}
