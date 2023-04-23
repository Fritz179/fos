use std::{process::Command, path::Path, env};

fn main() {
    if cfg!(not(target_os = "linux")) {
        panic!("Not on linux!")
    }

    println!("cargo:warning=Rebuilding WASM...");

    let out_dir = env::var("OUT_DIR").unwrap();

    println!("cargo:warning={}", out_dir);

    // Should pipe to stdout
    // Command::new("wasm-pack")
    //         // .args(["build", "../../wasm", "--target", "web"])
    //         // .args(["build", "./wasm", "--target", "web", "--out-dir", "../server/src/home/wasm"])
    //         .args(["build", "../wasm", "--target", "web", "--out-dir", &format!("{out_dir}/out")])
    //         .current_dir(&Path::new(&out_dir))
    //         .status()
    //         .expect("failed to build wasm");

    println!("cargo:warning={}", out_dir);
}

// include!(concat!(env!("OUT_DIR"), "/hello.rs"));