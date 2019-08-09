use opencontainers::image::ImagePlatformSelector;
use opencontainers::Registry;

fn main() {
    pretty_env_logger::init();

    let registry = Registry::new("https://registry-1.docker.io");
    let image = registry
        .image::<ImagePlatformSelector>("library/hello-world", "latest")
        .expect("Could not get image");

    println!("{:#?}", image.manifest());
    println!("{:#?}", image.config());

    for layer in image.manifest().layers().expect("could not get layers") {
        for entry in image.get_layer(layer).unwrap().entries().unwrap() {
            println!("{:?}", entry.unwrap().path());
        }
    }
}
