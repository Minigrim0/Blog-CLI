# Blog CLI
A CLI tool for managing blog posts
![crates.io badge](https://img.shields.io/crates/v/blog.svg)

## Features
* [X] Creation of blog posts directories with basic files
* [X] Adding/Removing/Listing tags from a post
* [X] Adding/Removing/Listing keywords from a post
* [X] Automatic fetch of header images from pexel using post's keywords
* [X] Management of the header images for the post
* [ ] Building a post (producing basic html, incluedable in other static sites)
* [ ] Deploying of the post (push of a zip) to a remote location with authentication option

## Usage

```bash
$ blog -h
A CLI blog post manager

Usage: blog <COMMAND>

Commands:
  new      Creates a new blog post with the given title
  build    Builds the blog post (fetches header images, generates index.html, etc.)
  publish  Publishes the blog post (Not implemented yet, missing remote handler)
  tag      Manages tags for a blog post
  keyword  Manages keywords for a blog post
  header   Manages header image for a blog post
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Blog posts
Upon creation of a new blog post, a tree of directories and files is created. The structure is as follows:
```
<year>
└── <month>
      └── <slugified-title>
          ├── content.md
          ├── metadata.toml
          ├── images/
```

metadata.toml contains the metadata of the post such as the publication and update dates, keywords and tags.

## Pre-commit hook
A pre-commit hook script is located in `.github/pre-commit`. It checks that the code is formatted with `rustfmt`, that `clippy` is happy and that the tests pass. To install
the hook, run the following command:
```bash
$ ln -s ../../.github/pre-commit .git/hooks/pre-commit
```
