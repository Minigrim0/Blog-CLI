use chrono::{Datelike, Utc};

use crate::post::Post;

#[test]
pub fn test_add_keyword() {
    let mut post = Post::new("Test post");

    // Test adding a keyword
    let result = post.metadata.opengraph.add_keyword("test".to_string());
    assert!(result.is_ok());
    assert_eq!(post.metadata.opengraph.keywords, vec!["test".to_string()]);

    // Test inserting the same keyword again
    let result = post.metadata.opengraph.add_keyword("test".to_string());
    assert!(result.is_err());
    assert_eq!(post.metadata.opengraph.keywords, vec!["test".to_string()]);

    // Test adding another keyword
    let result = post.metadata.opengraph.add_keyword("another".to_string());
    assert!(result.is_ok());
    assert_eq!(
        post.metadata.opengraph.keywords,
        vec!["test".to_string(), "another".to_string()]
    );
}

#[test]
pub fn test_remove_keyword() {
    let mut post = Post::new("Test post");

    // Test adding a keyword
    let result = post.metadata.opengraph.add_keyword("test".to_string());
    assert!(result.is_ok());
    assert_eq!(post.metadata.opengraph.keywords, vec!["test".to_string()]);

    // Test removoing non-existing keyword
    let result = post.metadata.opengraph.remove_keyword("idontexist");
    assert!(result.is_err());
    assert_eq!(post.metadata.opengraph.keywords, vec!["test".to_string()]);

    // Test adding another keyword
    let result = post.metadata.opengraph.remove_keyword("test");
    assert!(result.is_ok());
    let expected: Vec<String> = Vec::new();
    assert_eq!(post.metadata.opengraph.keywords, expected);
}

#[test]
pub fn test_add_tag() {
    let mut post = Post::new("Test post");

    // Test adding a keyword
    let result = post.metadata.post.add_tag("test".to_string());
    assert!(result.is_ok());
    assert_eq!(post.metadata.post.tags, vec!["test".to_string()]);

    // Test inserting the same keyword again
    let result = post.metadata.post.add_tag("test".to_string());
    assert!(result.is_err());
    assert_eq!(post.metadata.post.tags, vec!["test".to_string()]);

    // Test adding another keyword
    let result = post.metadata.post.add_tag("another".to_string());
    assert!(result.is_ok());
    assert_eq!(
        post.metadata.post.tags,
        vec!["test".to_string(), "another".to_string()]
    );
}

#[test]
pub fn test_remove_tag() {
    let mut post = Post::new("Test post");

    // Test adding a keyword
    let result = post.metadata.post.add_tag("test".to_string());
    assert!(result.is_ok());
    assert_eq!(post.metadata.post.tags, vec!["test".to_string()]);

    // Test removoing non-existing keyword
    let result = post.metadata.post.remove_tag("idontexist");
    assert!(result.is_err());
    assert_eq!(post.metadata.post.tags, vec!["test".to_string()]);

    // Test adding another keyword
    let result = post.metadata.post.remove_tag("test");
    assert!(result.is_ok());
    let expected: Vec<String> = Vec::new();
    assert_eq!(post.metadata.post.tags, expected);
}

#[test]
pub fn test_post_path() {
    let timestamp = Utc::now();

    let post = Post::new("Test");
    assert!(post.path.to_str().is_some());
    assert!(post
        .path
        .to_str()
        .unwrap()
        .contains(&format!("{:04}", timestamp.year())));
    assert!(post
        .path
        .to_str()
        .unwrap()
        .contains(&format!("{:02}", timestamp.month())));
    assert!(post.path.ends_with("test"));
}
