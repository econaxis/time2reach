FROM debian:12 AS base
ENV DEBIAN_FRONTEND=noninteractive


WORKDIR /app

COPY deploy/init.sh deploy/
RUN sh deploy/init.sh
RUN apt-get install -y build-essential libgdal-dev curl ca-certificates sqlite3 cmake pkg-config libclang-dev --no-install-recommends

FROM base as chef

COPY deploy/deploy.sh deploy/
RUN sh deploy/deploy.sh
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install cargo-chef

FROM chef as planner

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY gtfs-structure gtfs-structure
COPY gtfs-structure-2 gtfs-structure-2

COPY src src

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json --features prod
# Build application
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY gtfs-structure gtfs-structure
COPY gtfs-structure-2 gtfs-structure-2
RUN cargo build --release --features prod

FROM base AS run
WORKDIR /app
COPY city-gtfs /app/city-gtfs
COPY certificates /app/certificates
COPY web/public /app/web/public
COPY --from=builder /app/target/release/timetoreach /app/target/release/timetoreach

ENV RUST_LOG info,timetoreach=debug,h2=info,hyper=info,warp=info,rustls=info
ENTRYPOINT ["/app/target/release/timetoreach"]