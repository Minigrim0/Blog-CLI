use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use slugify::slugify;

use crate::header::{get_new_candidates, PexelPicture};
use crate::utils::{create_path, copy_dir_all};

#[derive(Debug)]
/// A blog post, represented on disk by a minimum of two files,
/// * content.md  # The content of the file
/// * metadata.toml  # The post's metadata
pub struct Post {
    pub content: String,    // Markdown content
    pub path: PathBuf,      // Path to the post
    pub metadata: Metadata, // Metadata of the post
}

impl Post {
    /// Creates a new post with the given title.
    pub fn new<S: AsRef<str>>(title: S) -> Self {
        let title = title.as_ref().to_string();
        info!("Creating new post with title: {}", title);

        let path = {
            let today = Utc::now();
            let mut path = PathBuf::new();
            path.push(format!("{:04}", today.year()));
            path.push(format!("{:02}", today.month()));
            path.push(slugify!(title.as_str()));
            path
        };
        info!(
            "Generated path: {}",
            path.to_str().unwrap_or("Error; unable to display path")
        );

        Self {
            content: format!("# {title}"),
            path,
            metadata: Metadata::default().with_title(title),
        }
    }

    /// Tries to load a post from the given path.
    pub fn load(path: String) -> Result<Self, String> {
        info!("Loading post from path: {}", path);
        let path = PathBuf::from(path);
        if !path.exists() {
            error!(
                "Path does not exist: {}",
                path.to_str().unwrap_or("Error; unable to display path")
            );
            return Err("Blog post does not exist".to_string());
        }

        let content_path = path.join(Path::new("content.md"));
        let content = fs::read_to_string(&content_path)
            .map_err(|e| format!("Failed to read content file: {e}"))?;

        let metadata_path = path.join(Path::new("metadata.toml"));
        let metadata_toml = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read metadata file: {e}"))?;

        let metadata: Metadata = toml::from_str(&metadata_toml)
            .map_err(|e| format!("Failed to parse metadata file: {e}"))?;

        Ok(Self {
            content,
            path,
            metadata,
        })
    }

    /// Builds the post, creating the output directory and writing the post's content to an index.html file.
    /// It will also update the post's metadata file with the current date and time.
    pub fn build(&mut self) -> Result<(), String> {
        self.metadata.post.update = Some(Utc::now());
        self.save()?;

        let output_path: PathBuf = self.path.join(Path::new("dist/"));
        info!(
            "Building post at path: {}",
            output_path
                .to_str()
                .unwrap_or("Error; unable to display path")
        );

        create_path(&output_path)?;

        let html_content = markdown::to_html_with_options(&self.content, &markdown::Options::gfm()).map_err(|e| e.to_string())?;

        let output_file = output_path.join(Path::new("index.html"));
        fs::write(&output_file, html_content)
            .map_err(|e| format!("Failed to write output file: {e}"))?;

        // Copy images folder
        let images_path = self.path.join(Path::new("images"));
        let output_images_path = output_path.join(Path::new("images"));
        copy_dir_all(&images_path, &output_images_path)
            .map_err(|e| format!("Failed to copy images folder: {e}"))?;

        Ok(())
    }

    #[allow(clippy::unused_self)]
    /// Publishes the post, uploading it to the blog's server.
    pub fn publish(&mut self) -> Result<(), String> {
        Err("Not implemented".to_string())
    }

    /// Saves the post to disk.
    pub fn save(&self) -> Result<(), String> {
        create_path(&self.path)?;
        let images_path = self.path.join("images");
        create_path(&images_path)?;

        let content_path = format!("{}/content.md", self.path_display());
        fs::write(&content_path, &self.content)
            .map_err(|e| format!("Failed to write content file: {e}"))?;

        let metadata_path = format!("{}/metadata.toml", self.path_display());
        let metadata_toml = toml::to_string(&self.metadata)
            .map_err(|e| format!("Failed to serialize metadata: {e}"))?;

        fs::write(&metadata_path, metadata_toml)
            .map_err(|e| format!("Failed to write metadata file: {e}"))?;

        Ok(())
    }

    /// Returns a string representation of the post's path. Or an error message if the path is invalid.
    fn path_display(&self) -> String {
        self.path
            .to_str()
            .unwrap_or("Error; unable to display path")
            .to_string()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub post: PostInfo,
    pub opengraph: OpenGraph,
}

impl Metadata {
    pub fn with_title<S: AsRef<str>>(mut self, title: S) -> Self {
        self.post.title = title.as_ref().to_string();
        self
    }
}

impl Metadata {
    pub fn header_path(blog_path: &Path) -> PathBuf {
        let header_sub_path: PathBuf = [r"images", "header"].iter().collect();
        blog_path.join(header_sub_path)
    }

    pub fn header_exists(path: &Path) -> Option<PathBuf> {
        let mut header_path = Self::header_path(path);
        header_path.push("header.jpg");
        if header_path.exists() && header_path.is_file() {
            Some(header_path)
        } else {
            None
        }
    }

    /// Fetches new candidate header images from pexel
    pub fn fetch_new_header_images(&self, path: &Path, amount: usize) -> Result<(), String> {
        if self.opengraph.keywords.is_empty() {
            return Err(
                "Unable to fetch image for the blog post; The post has no keyword".to_string(),
            );
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        let _ = rt.block_on(get_new_candidates(
            Self::header_path(path),
            &self.opengraph.keywords,
            amount,
        ))?;

        Ok(())
    }

    pub fn list_header_candidates(path: &Path) -> Result<(), String> {
        let header_path = Self::header_path(path).join("candidates");

        let mut index = 1;
        for path in fs::read_dir(header_path).map_err(|e| e.to_string())? {
            let path = path.map_err(|e| e.to_string())?;
            if let Some(extension) = path.path().extension() {
                if extension == "toml" {
                    let content = fs::read_to_string(path.path()).map_err(|e| e.to_string())?;
                    let picture = toml::from_str::<PexelPicture>(content.as_str())
                        .map_err(|e| e.to_string())?;
                    println!("{index} - {picture}");

                    index += 1;
                }
            }
        }

        Ok(())
    }

    pub fn choose_header(path: &Path, index: usize) -> Result<(), String> {
        if Self::header_exists(path).is_some() {
            warn!("A header file has already been selected, it will be overwritten");
        }

        let header_path = Self::header_path(path);

        let chosen_header_picture = header_path.join("header.jpg");
        let chosen_header_metadata = header_path.join("header.toml");

        let candidate_path = header_path.join("candidates");
        let candidate_header_picture = candidate_path.join(format!("header_{index}.jpg"));
        let candidate_header_metadata = candidate_path.join(format!("header_{index}.toml"));

        if !candidate_header_picture.exists() || !candidate_header_picture.is_file() {
            return Err(format!(
                "No candidate header with the id {index} could be found",
            ));
        }
        if !candidate_header_metadata.exists() || !candidate_header_metadata.is_file() {
            return Err(format!(
                "The metadata file for candidate header {index} could not be found",
            ));
        }

        // Move header picture & metadata one folder above
        fs::copy(candidate_header_picture, chosen_header_picture).map_err(|e| e.to_string())?;
        fs::copy(candidate_header_metadata, chosen_header_metadata).map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PostInfo {
    pub title: String,
    pub author: String,
    pub published_date: Option<DateTime<Utc>>,
    pub update: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

impl PostInfo {
    /// Adds a tag to the post.
    pub fn add_tag(&mut self, tag: String) -> Result<(), String> {
        info!("Adding tag {tag} to post");
        if self.tags.contains(&tag) {
            Err(format!("Tag `{tag}` is already attached to this blog post",))
        } else {
            self.tags.push(tag);
            Ok(())
        }
    }

    /// Removes a tag from the post.
    pub fn remove_tag(&mut self, tag: &str) -> Result<(), String> {
        info!("Removing tag {tag} from post");
        if self.tags.contains(&tag.to_string()) {
            let index = self
                .tags
                .iter()
                .position(|x| x == tag)
                .ok_or(format!("Tag `{tag}` was not found in the post's tags"))?;
            self.tags.remove(index);
            Ok(())
        } else {
            Err(format!("Tag `{tag}` is already attached to this blog post",))
        }
    }

    /// Lists the tags attached to the post.
    pub fn list_tags(&self) {
        if self.tags.is_empty() {
            println!("This post has no tags");
            return;
        }

        for tag in &self.tags {
            println!("* {tag}");
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct OpenGraph {
    pub short: String,
    pub opengraphimage: String,
    pub description: String,
    pub keywords: Vec<String>,
}

impl OpenGraph {
    /// Adds a tag to the post.
    pub fn add_keyword(&mut self, keyword: String) -> Result<(), String> {
        info!("Adding keyword {} to post", keyword);
        if self.keywords.contains(&keyword) {
            Err(format!(
                "Keyword `{keyword}` is already attached to this blog post"
            ))
        } else {
            self.keywords.push(keyword);
            Ok(())
        }
    }

    /// Removes a keyword from the post.
    pub fn remove_keyword(&mut self, keyword: &str) -> Result<(), String> {
        info!("Removing keyword {} from post", keyword);
        if self.keywords.contains(&keyword.to_string()) {
            let index = self
                .keywords
                .iter()
                .position(|x| x == keyword)
                .ok_or(format!(
                    "Keyword `{keyword}` was not found in the post's tags",
                ))?;
            self.keywords.remove(index);
            Ok(())
        } else {
            Err(format!(
                "Keyword `{keyword}` is already attached to this blog post",
            ))
        }
    }

    /// Lists the tags attached to the post.
    pub fn list_keywords(&self) {
        if self.keywords.is_empty() {
            println!("This post has no keywords");
            return;
        }

        for keyword in &self.keywords {
            println!("* {keyword}");
        }
    }
}
