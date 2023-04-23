extern crate rouille;

use rouille::Response;

fn main() {
    // let dir = std::env::current_dir().unwrap();
    // let dir = dir.to_str().unwrap();

    // println!("{dir}");

    println!("Now listening on localhost:8000");

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