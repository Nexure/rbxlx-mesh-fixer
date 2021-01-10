use std::error::Error;

pub mod asset_downloader;
pub mod cframe;
pub mod mesh_reader;

type GenericError = Box<dyn Error + 'static>;
