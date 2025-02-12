use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::var;
use std::path::PathBuf;
use reqwest;
use log::info;

use crate::post::Post;
use crate::utils::create_path;

#[derive(Deserialize)]
struct PexelResponse {
    pub photos: Vec<PexelPicture>
}

#[derive(Deserialize, Serialize)]
pub struct PexelPicture {
    width: usize,
    height: usize,
    url: String,
    photographer: String,
    photographer_url: String,
    src: HashMap<String, String>
}

pub async fn get_image(post: &Post, limit: usize) -> Result<Vec<PathBuf>, String> {
    dotenv().ok();

    let pexel_api_key = var("PEXEL_API_KEY").map_err(|_| "Missing PEXEL_API_KEY".to_string())?;

    let client = reqwest::Client::new();
    info!("Fetching image from pexel for post: {}", post.path.display());
    let response = client.get("https://api.pexels.com/v1/search")
        .header("Authorization", pexel_api_key)
        .query(&[("query", post.get_keywords().join(", "))])
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
                info!("[{:3}/{:3}] Fetching image: {}", index + 1, pexel_response.photos.len(), image_url);
                let image_response = client.get(image_url)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                let image_bytes = image_response.bytes().await.map_err(|e| e.to_string())?;
                let base_path = post.path.join("images").join("header");
                let image_path = base_path.join(format!("header_{index}.jpg"));
                let image_metadata = base_path.join(format!("header_{index}.toml"));

                create_path(&base_path)?;

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
