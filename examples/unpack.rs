use opencontainers::glue::{Unpack, UnpackError};
use opencontainers::image::TestImageSelector as ImagePlatformSelector;
use opencontainers::Registry;
use std::path::Path;

struct Extractor {}

impl Extractor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Unpack for Extractor {
    fn add<R: std::io::Read>(&self, entry: tar::Entry<R>) -> Result<(), UnpackError> {
        let path: std::path::PathBuf = entry.path().map_err(UnpackError::GetEntryPath)?.into();
        println!("  Would extract path: {}", path.to_string_lossy());
        Ok(())
    }

    fn whiteout_file<P: AsRef<Path>>(&self, path: P) -> Result<(), UnpackError> {
        println!("  Would whiteout path: {}", path.as_ref().to_string_lossy());
        Ok(())
    }

    fn whiteout_folder<P: AsRef<Path>>(&self, path: P) -> Result<(), UnpackError> {
        println!(
            "  Would whiteout all children of: {}",
            path.as_ref().to_string_lossy()
        );
        Ok(())
    }

    fn pre_apply(&self) -> Result<(), UnpackError> {
        println!("Starting to extract new layer");
        Ok(())
    }

    fn post_apply(&self) -> Result<(), UnpackError> {
        println!("Done extracting layer");
        Ok(())
    }
}

fn main() {
    pretty_env_logger::init();

    let registry = Registry::new("https://registry-1.docker.io");
    let image = registry
        .image::<ImagePlatformSelector>("fubarnetes/whiteout-test", "latest")
        .expect("Could not get image");

    let extractor = Extractor::new();
    extractor.unpack(&image).unwrap();
}
