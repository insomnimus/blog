# Blog
A simplistic blogging webserver with a custom tooling.

## Disclaimer
This project is not meant to be a general solution and is mainly for showcasing and sharing code for the sake of it; in short, it's personal (for now).

## Building The Project (development)
## Docker (server only)
There's a convenience docker image provided, you can run it with `docker compose up -d`.

After this, visiting `localhost:3000` on a browser will get you the home page.

You still need to populate the database for the content, for that you need to build the command line tooling as well, see below.

## Native Build (server and tooling)
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

