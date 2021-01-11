use std::error::Error;

pub mod asset_downloader;
pub mod cframe;
pub mod mesh_reader;

pub type GenericError = Box<dyn Error + 'static>;
type TupleComponent = (
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
);
