extern crate rouille;

use std::process::Command;

use rouille::Response;

fn main() {
    // let dir = std::env::current_dir().unwrap();
    // let dir = dir.to_str().unwrap();

    // println!("{dir}");

    Command::new("wasm-pack")
        // .args(["build", "../../wasm", "--target", "web"])
        .args([
            "build",
            "./wasm",
            "--target",
            "web",
            "--out-dir",
            "../server/src/home/wasm",
        ])
        // .args(["build", "../wasm", "--target", "web", "--out-dir", ])
        .status()
        .expect("failed to build wasm");

    println!("Visit `http://localhost:8000/index.html`");

    rouille::start_server("localhost:8000", move |request| {
        let response = rouille::match_assets(request, "./server/src/home");

        if response.is_success() {
            return response;
        }

        Response::html(
            "404 error. Try <a href=\"/README.md\"`>README.md</a> or \
                        <a href=\"/src/lib.rs\">src/lib.rs</a> for example.",
        )
        .with_status_code(404)
    });
}
