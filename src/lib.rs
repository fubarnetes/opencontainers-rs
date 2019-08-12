#[macro_use]
extern crate failure;

#[macro_use]
extern crate log;

#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate serde_derive;

pub mod distribution;
pub use distribution::Registry;

pub mod image;
pub use image::Image;

pub mod runtime;
pub use runtime::{Bundle, Runtime};

pub mod glue;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Registry Error: {}", _0)]
    RegistryError(distribution::RegistryError),

    #[fail(display = "Error unpacking: {}", _0)]
    UnpackError(glue::UnpackError),
}
