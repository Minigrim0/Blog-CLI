use std::fs;
use std::path::{Path, PathBuf};

use log::{info, error};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Datelike, Utc};
use slugify::slugify;

use crate::utils::create_path;

#[derive(Debug)]
pub struct Post {
    pub content: String,  // Markdown content
    pub path: PathBuf,     // Path to the post
    pub metadata: Metadata,
}

impl Post {
    /// Creates a new post with the given title.
    pub fn new(title: String) -> Self {
        info!("Creating new post with title: {}", title);

        let path = {
            let today = Utc::now();
            let mut path = PathBuf::new();
            path.push(format!("{:04}", today.year()));
            path.push(format!("{:02}", today.month()));
            path.push(slugify!(title.as_str()));
            path
        };
        info!("Generated path: {}", path.to_str().unwrap_or("Error; unable to display path"));

        Self {
            content: format!("# {}", title),
            path,
            metadata: Metadata::default().with_title(title),
        }
    }

    /// Tries to load a post from the given path.
    pub fn load(path: String) -> Result<Self, String> {
        info!("Loading post from path: {}", path);
        let path = PathBuf::from(path);
        if !path.exists() {
            error!("Path does not exist: {}", path.to_str().unwrap_or("Error; unable to display path"));
            return Err("Blog post does not exist".to_string());
        }

        let content_path = path.join(Path::new("content.md"));
        let content = fs::read_to_string(&content_path)
            .map_err(|e| format!("Failed to read content file: {}", e))?;

        let metadata_path = path.join(Path::new("metadata.toml"));
        let metadata_toml = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read metadata file: {}", e))?;

        let metadata: Metadata = toml::from_str(&metadata_toml)
            .map_err(|e| format!("Failed to parse metadata file: {}", e))?;

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
        info!("Building post at path: {}", output_path.to_str().unwrap_or("Error; unable to display path"));

        create_path(&output_path)?;

        let html_content = markdown::to_html(&self.content);

        let output_file = output_path.join(Path::new("index.html"));
        fs::write(&output_file, html_content)
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        Ok(())
    }

    /// Publishes the post, uploading it to the blog's server.
    pub fn publish(&mut self) -> Result<(), String> {
        Err("Not implemented".to_string())
    }

    /// Saves the post to disk.
    pub fn save(&self) -> Result<(), String> {
        create_path(&self.path)?;

        let content_path = format!("{}/content.md", self.path_display());
        fs::write(&content_path, &self.content)
            .map_err(|e| format!("Failed to write content file: {}", e))?;

        let metadata_path = format!("{}/metadata.toml", self.path_display());
        let metadata_toml = toml::to_string(&self.metadata)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        fs::write(&metadata_path, metadata_toml)
            .map_err(|e| format!("Failed to write metadata file: {}", e))?;

        Ok(())
    }

    pub fn update_header_image(&self) -> Result<(), String> {
        let mut header_path = {
            let header_sub_path: PathBuf = [r"images", "header"].iter().collect();
            self.path.join(header_sub_path)
        };

        if header_path.exists() {
            header_path.push("header.jpg");
            if header_path.exists() && header_path.is_file() {
                return Err("A header file already exists for this blog post. If you want to find a new one, delete this file and run the command again".to_string());
            }
        }

        if self.metadata.opengraph.keywords.is_empty() {
            return Err("Unable to fetch image for the blog post; The post has no keyword".to_string());
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        let _ = rt.block_on(super::pexel::get_image(self, 3))?;

        Ok(())
    }

    /// Returns the blog post keywords.
    /// ! This copies the tags vector
    pub fn get_keywords(&self) -> Vec<String> {
        self.metadata.opengraph.keywords.clone()
    }

    /// Returns a string representation of the post's path. Or an error message if the path is invalid.
    fn path_display(&self) -> String {
        self.path.to_str().unwrap_or("Error; unable to display path").to_string()
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PostInfo {
    pub title: String,
    pub author: String,
    pub published_date: Option<DateTime<Utc>>,
    pub update: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

impl Default for PostInfo {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            published_date: None,
            update: None,
            author: "".to_string(),
            tags: vec![],
        }
    }
}

impl PostInfo {
    /// Adds a tag to the post.
    pub fn add_tag(&mut self, tag: String) -> Result<(), String> {
        info!("Adding tag {} to post", tag);
        if self.tags.contains(&tag) {
            Err(format!("Tag `{}` is already attached to this blog post", tag))
        } else {
            self.tags.push(tag);
            Ok(())
        }
    }

    /// Removes a tag from the post.
    pub fn remove_tag(&mut self, tag: String) -> Result<(), String> {
        info!("Removing tag {} from post", tag);
        if !self.tags.contains(&tag) {
            Err(format!("Tag `{}` is already attached to this blog post", tag))
        } else {
            let index = self.tags.iter().position(|x| x == &tag).ok_or(format!("Tag `{}` was not found in the post's tags", tag))?;
            self.tags.remove(index);
            Ok(())
        }
    }

    /// Lists the tags attached to the post.
    pub fn list_tags(&self) {
        if self.tags.is_empty() {
            println!("This post has no tags");
            return;
        }

        for tag in &self.tags {
            println!("* {}", tag);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenGraph {
    pub short: String,
    pub opengraphimage: String,
    pub description: String,
    pub keywords: Vec<String>,
}

impl Default for OpenGraph {
    fn default() -> Self {
        Self {
            short: "".to_string(),
            description: "".to_string(),
            opengraphimage: "".to_string(),
            keywords: vec![]
        }
    }
}

impl OpenGraph {
    /// Adds a tag to the post.
    pub fn add_keyword(&mut self, keyword: String) -> Result<(), String> {
        info!("Adding keyword {} to post", keyword);
        if self.keywords.contains(&keyword) {
            Err(format!("Keyword `{}` is already attached to this blog post", keyword))
        } else {
            self.keywords.push(keyword);
            Ok(())
        }
    }

    /// Removes a keyword from the post.
    pub fn remove_keyword(&mut self, keyword: String) -> Result<(), String> {
        info!("Removing keyword {} from post", keyword);
        if !self.keywords.contains(&keyword) {
            Err(format!("Keyword `{}` is already attached to this blog post", keyword))
        } else {
            let index = self.keywords.iter().position(|x| x == &keyword).ok_or(format!("Keyword `{}` was not found in the post's tags", keyword))?;
            self.keywords.remove(index);
            Ok(())
        }
    }

    /// Lists the tags attached to the post.
    pub fn list_keywords(&self) {
        if self.keywords.is_empty() {
            println!("This post has no keywords");
            return;
        }

        for keyword in &self.keywords {
            println!("* {}", keyword);
        }
    }
}
