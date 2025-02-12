/// This module handles the automatic download of images from the Pexel API.
/// It fetches images based on the keywords associated with a post, saves the images
/// and their metadata to the filesystem, and manages the organization of these images.
/// The main functionality includes:
///
/// - Fetching images from the Pexel API using a specified limit.
/// - Saving the downloaded images and their metadata in a structured format.
/// - Ensuring the required environment variables are set for API access.
/// - Logging the process of fetching and saving images for debugging and tracking purposes.

use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::var;
use std::fmt;
use std::path::PathBuf;
use reqwest;
use log::info;

use crate::utils::create_path;

#[derive(Deserialize)]
/// The structure of the response from the pexel API
struct PexelResponse {
    pub photos: Vec<PexelPicture>
}

#[derive(Deserialize, Serialize)]
/// The structure of a picture from the pexel API
/// This structure is saved in a TOML file along with the image
pub struct PexelPicture {
    width: usize,
    height: usize,
    url: String,
    photographer: String,
    photographer_url: String,
    src: HashMap<String, String>,
    alt: String
}

impl fmt::Display for PexelPicture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Picture by {} - {} `{}`", self.photographer, self.url, self.alt)
    }
}

/// Fetches the requested number of images from the pexel API.
/// This requires the PEXEL_API_KEY to be set in the environment.
///
/// This function returns a vector containing the paths to all the new images or an error
pub async fn get_new_candidates(path: PathBuf, keywords: &Vec<String>, limit: usize) -> Result<Vec<PathBuf>, String> {
    dotenv().ok();

    let pexel_api_key = var("PEXEL_API_KEY").map_err(|_| "Missing PEXEL_API_KEY".to_string())?;
    let candidates_paths = path.join("candidates");
    create_path(&candidates_paths)?;

    let client = reqwest::Client::new();
    info!("Fetching image from pexel for post: {}", path.display());
    let response = client.get("https://api.pexels.com/v1/search")
        .header("Authorization", pexel_api_key)
        .query(&[("query", keywords.join(", "))])
        .query(&[("per_page", limit.to_string().as_str())])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let pexel_response = response.json::<PexelResponse>().await.map_err(|e| e.to_string())?;
            let mut images = vec![];

            for (index, image) in pexel_response.photos.iter().enumerate() {
                let image_url = image.src.get("landscape").ok_or("Unable to retreive landscape image from pexel picture".to_string())?;
                let image_path = candidates_paths.join(format!("header_{}.jpg", index + 1));
                let image_metadata = candidates_paths.join(format!("header_{}.toml", index + 1));

                info!("[{:3}/{:3}] Fetching image: {}", index + 1, pexel_response.photos.len(), image_url);
                let image_response = client.get(image_url)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                let image_bytes = image_response.bytes().await.map_err(|e| e.to_string())?;

                std::fs::write(&image_path, image_bytes).map_err(|e| e.to_string())?;
                let image_metadata_toml = toml::to_string(&image).map_err(|e| e.to_string())?;
                std::fs::write(&image_metadata, image_metadata_toml).map_err(|e| e.to_string())?;

                images.push(image_path);
            }

            Ok(images)
        }
        _ => {
            Err(format!("Failed to fetch image: {}", response.text().await.map_err(|e| e.to_string())?))
        }
    }
}
