extern crate opencontainers;
extern crate pretty_env_logger;

use opencontainers::Registry;

fn main() {
    pretty_env_logger::init();

    let registry = Registry::new("https://registry-1.docker.io");
    let manifest = registry.manifest("hello-world", "latest").unwrap();

    println!("{:?}", manifest);
}
