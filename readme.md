# Blog
A simplistic blogging webserver with a custom tooling.

## Disclaimer
This project is not meant to be a general solution and is mainly for showcasing and sharing code for the sake of it; in short, it's personal (for now).

## Building The Project (development)
You will need several things.
-	Docker (compose) to get the development db up and running.
-	Cargo, the rust languages build system to build the project.

Build and run the postgresql image:

```sh
cd pg
docker compose up -d
cd ..
```

Build the entire project (both the server and the cli):

```sh
cargo build --release
```

> Set the `BLOG_DB_URL` environment variable to `postgres://blog:blog@localhost:59595/blog`.
> Or pass the same value to the blog webserver and cli.
> This value is for the development version of the postgresql image we just built and ran.

To start the server:
```sh
# NOTE: Make sure the current working directory contains the `static` folder.
./target/release/blog-server
```

For how to publish articles/posts, please run

`./target/release/blog help`
