# Blog
A simplistic blogging webserver with a custom tooling.

## Disclaimer
This project is not meant to be a general solution and is mainly for showcasing and sharing code for the sake of it; in short, it's personal (for now).

## Building The Project (development)
You will need several things.
-	Docker (compose) to get the development db up and running.
-	Cargo, the rust languages build system to build the project.
-	An active database for the compile-time checked queries to work.

After cloning the project, set these environment variables:

-	`DATABASE_URL`: `postgres://blog:blog@localhost:59595/blog`
-	`BLOG_DB_URL`: `postgres://blog:blog@localhost:59595/blog`

Now, build and run the postgresql image:

```sh
cd pg
docker compose up -d
```

Build the entire project (both the server and the cli):

```sh
cargo build --release
```

To start the server:
`./target/release/blog-server`

To publish/list articles:
`./target/release/blog --help`

