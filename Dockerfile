FROM rust:1.58-buster AS prep
WORKDIR app
RUN cargo install cargo-chef

FROM prep AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM prep AS build
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --bin blog-server --release

FROM debian:buster-slim AS binary

RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends -y \
	libssl-dev \
	ca-certificates

WORKDIR app
COPY static static
COPY --from=build /app/target/release/blog-server blog-server

CMD ["/app/blog-server"]
