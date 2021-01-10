use std::{
    fs::{metadata, File},
    io::{self, Cursor, Read},
    path::Path,
};

use regex::Regex;

use super::GenericError;

pub fn extract_assetid(asset_id: String) -> Result<String, GenericError> {
    let regex = Regex::new(r"(?m)(\d+)")?;
    let result = regex.find(&asset_id).ok_or("Invalid regex")?;
    Ok(result.as_str().to_string())
}

pub fn download_asset(asset_id: String) -> Result<Cursor<Vec<u8>>, GenericError> {
    let extracted_asset_id = extract_assetid(asset_id)?;
    let asset_path = format!("cache/{}", extracted_asset_id);
    let asset_url = format!(
        "https://assetdelivery.roblox.com/v1/asset?id={}",
        extracted_asset_id
    );

    let path = Path::new(&asset_path);
    if !metadata(path).is_ok() {
        let mut response = reqwest::blocking::get(&asset_url)?;
        assert!(response.status().is_success());

        let mut file = File::create(path).expect("Unable to cache file");
        io::copy(&mut response, &mut file)?;
    }

    let mut file = File::open(path)?;
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;
    Ok(Cursor::new(buffer))
}
