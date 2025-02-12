use clap::Parser;

#[derive(Parser)]
#[clap(name = "blog")]
#[clap(version)]
/// A CLI blog post manager
pub struct Cli {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    #[clap(name = "new")]
    /// Creates a new blog post with the given title
    New { title: String },
    #[clap(name = "build")]
    /// Builds the blog post (fetches header images, generates index.html, etc.)
    Build { path: String },
    #[clap(name = "publish")]
    /// Publishes the blog post (Not implemented yet, missing remote handler)
    Publish { path: String },
    #[clap(name = "tag")]
    /// Manages tags for a blog post
    Tag(Tag),
    #[clap(name = "keyword")]
    /// Manages keywords for a blog post
    Keyword(Keyword),
    #[clap(name = "header")]
    /// Manages header image for a blog post
    Header(Header),
}

#[derive(Parser)]
pub struct Tag {
    /// The path to the post
    pub post: String,
    #[clap(subcommand)]
    pub subcmd: TagSubCommand,
}

#[derive(Parser)]
pub enum TagSubCommand {
    #[clap(name = "add")]
    /// Adds the space separated tags to the post
    Add { tags: Vec<String> },
    #[clap(name = "remove")]
    /// Removes the space separated tags from the post
    Remove { tags: Vec<String> },
    #[clap(name = "list")]
    /// Lists the tags attached to the post
    List,
}

#[derive(Parser)]
pub struct Keyword {
    /// The path to the post
    pub post: String,
    #[clap(subcommand)]
    pub subcmd: KeywordSubCommand,
}

#[derive(Parser)]
pub enum KeywordSubCommand {
    #[clap(name = "add")]
    /// Adds the space separated keywords to the post
    Add { keywords: Vec<String> },
    #[clap(name = "remove")]
    /// Removes the space separated keywords from the post
    Remove { keywords: Vec<String> },
    #[clap(name = "list")]
    /// Lists the keywords attached to this post
    List,
}

#[derive(Parser)]
pub struct Header {
    /// The path to the post
    pub post: String,
    #[clap(subcommand)]
    pub subcmd: HeaderSubCommand,
}

#[derive(Parser)]
pub enum HeaderSubCommand {
    #[clap(name = "choose")]
    /// Chooses one of the proposed header images as the header image for the post
    Choose { index: usize },
    #[clap(name = "fetch")]
    /// Fetches header images from Pexel for the post
    Fetch { amount: usize },
    #[clap(name = "list")]
    /// Lists the header images paths for the post
    List,
}
