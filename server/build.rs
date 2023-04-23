use std::process::Command;

fn main() {
    if cfg!(not(target_os = "linux")) {
        panic!("Not on linux!")
    }

    println!("cargo:warning=Rebuilding WASM...");

    Command::new("wasm-pack")
            .args(["build", "../wasm", "--target", "web", "--out-dir", "../server/wasm/src/home"])
            .output()
            .expect("failed to build wasm");
}