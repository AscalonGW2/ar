use std::{env, path::PathBuf};

use ascalon_ar_gen::generate;

fn main() {
    println!("cargo:rerun-if-changed=packfile.json");

    generate(
        "packfile.json",
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("packfile.rs"),
    );
}
