use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=BUILD_SHA");
    let build_sha = env::var("BUILD_SHA").unwrap_or_else(|_| "dev".to_owned());
    println!("cargo:rustc-env=BUILD_SHA={build_sha}");
}
