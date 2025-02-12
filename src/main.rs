use clap::Parser;
use colog;

#[cfg(test)]
mod tests;

mod cli;
mod header;
mod post;
mod utils;

/// Handles the commands related to keywords
fn handle_keyword_command(command: cli::Keyword) {
    let mut post = post::Post::load(command.post)
        .unwrap_or_else(|e| {
            println!("Failed to load post: {}", e);
            std::process::exit(1);
        });

    match command.subcmd {
        cli::KeywordSubCommand::Add { keywords } => {
            for kw in keywords {
                if let Err(e) = post.metadata.opengraph.add_keyword(kw) {
                    println!("Unable to add keyword: {}", e);
                }
            }

            if let Err(e) = post.save() {
                println!("Unable to save post: {}", e);
            }
        }
        cli::KeywordSubCommand::Remove { keywords } => {
            for kw in keywords {
                if let Err(e) = post.metadata.opengraph.remove_keyword(kw) {
                    println!("Unable to remove keyword: {}", e);
                }
            }

            if let Err(e) = post.save() {
                println!("Unable to save post: {}", e);
            }
        }
        cli::KeywordSubCommand::List => {
            post.metadata.opengraph.list_keywords();
        }
    }
}

/// Handles the commands related to tags
fn handle_tag_command(command: cli::Tag) {
    let mut post = post::Post::load(command.post)
        .unwrap_or_else(|e| {
            println!("Failed to load post: {}", e);
            std::process::exit(1);
        });

    match command.subcmd {
        cli::TagSubCommand::Add { tags } => {
            for tag in tags {
                if let Err(e) = post.metadata.post.add_tag(tag) {
                    println!("Unable to add tag: {}", e);
                }
            }

            if let Err(e) = post.save() {
                println!("Unable to save post: {}", e);
            }
        }
        cli::TagSubCommand::Remove { tags } => {
            for tag in tags {
                if let Err(e) = post.metadata.post.remove_tag(tag) {
                    println!("Unable to remove tag: {}", e);
                }
            }

            if let Err(e) = post.save() {
                println!("Unable to save post: {}", e);
            }
        }
        cli::TagSubCommand::List => {
            post.metadata.post.list_tags();
        }
    }
}

fn handle_header_command(command: cli::Header) {
    let post = post::Post::load(command.post)
        .unwrap_or_else(|e| {
            println!("Failed to load post: {}", e);
            std::process::exit(1);
        });

    match command.subcmd {
        cli::HeaderSubCommand::Choose { index } => {
            if let Err(e) = post.metadata.choose_header(&post.path, index) {
                println!("Error while selecting the header: {}", e);
            }
        }
        cli::HeaderSubCommand::Fetch { amount } => {
            if let Err(e) = post.metadata.fetch_new_header_images(&post.path, amount) {
                println!("Error while fetching new posts: {}", e);
            }
        }
        cli::HeaderSubCommand::List => {
            if let Err(e) = post.metadata.list_header_candidates(&post.path) {
                println!("Error while displaying candidate pictures: {}", e);
            }
        }
    }
}

fn main() {
    colog::init();

    let args = cli::Cli::parse();

    match args.subcmd {
        cli::SubCommand::New { title } => {
            let post = post::Post::new(title);

            if let Err(e) = post.save() {
                println!("Failed to save post: {}", e);
            }
        }
        cli::SubCommand::Build { path } => {  // Building a post will create its output directory and write the post's content to an index.html file. It will also update the post's metadata file with the current date and time.
            let mut post = post::Post::load(path)
                .unwrap_or_else(|e| {
                    println!("Failed to load post: {}", e);
                    std::process::exit(1);
                });

            if let Err(e) = post.build() {
                println!("Failed to build post: {}", e);
            }
        }
        cli::SubCommand::Publish { path } => {
            println!("Publishing post: {}", path);
            let mut post = post::Post::load(path)
                .unwrap_or_else(|e| {
                    println!("Failed to load post: {}", e);
                    std::process::exit(1);
                });

            if let Err(e) = post.publish() {
                println!("Error while publishing post: {}", e);
            }
        }
        cli::SubCommand::Tag(command) => {
            handle_tag_command(command);
        }
        cli::SubCommand::Keyword(command) => {
            handle_keyword_command(command);
        }
        cli::SubCommand::Header(command) => {
            handle_header_command(command);
        }
    }
}
