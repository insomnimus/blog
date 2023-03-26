# Blog
This is the source code of my blog.

## Disclaimer
This project is not meant to be a general solution for everyone and is mainly for showcasing and sharing code for the sake of it; in short, it's personal.

## Documentation
[Visit this page](doc/index.md).

## Building it
Get a recent rust toolchain installed on your system and follow the instructions below.

1. Clone the repository.
  `$ git clone --depth 1 https://github.com/insomnimus/blog && cd blog`
2. Build the project.
  `$ cargo build --release`

Two binaries will be generated in `target/release/`: `blog-server` and `blog`.
On windows they will also have a `.exe` extension.

The `blog` binary is the command line content management utility and the `blog-server` binary is the web server.

For more information see the [documentation](doc/index.md).
