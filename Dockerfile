FROM rust:1.58-buster AS build

WORKDIR app
COPY . .
RUN cargo build --bin blog-server

FROM debian:buster-slim AS binary
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends -y \
	libssl-dev \
	ca-certificates

WORKDIR app
COPY static static
COPY --from=build /app/target/debug/blog-server blog-server

CMD ["/app/blog-server"]
