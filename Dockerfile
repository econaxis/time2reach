FROM debian:12 AS base
ENV DEBIAN_FRONTEND=noninteractive


WORKDIR /app

COPY deploy/init.sh deploy/
RUN sh deploy/init.sh
RUN apt-get install -y libgdal-dev curl ca-certificates sqlite3 --no-install-recommends

FROM base as chef

RUN apt-get install -y  build-essential pkg-config cmake libclang-dev libssl-dev --no-install-recommends

COPY rust-toolchain.toml rust-toolchain.toml
COPY deploy/deploy.sh deploy/
RUN sh deploy/deploy.sh
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install cargo-chef

FROM chef as planner

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY gtfs-structure gtfs-structure
COPY gtfs-structure-2 gtfs-structure-2
COPY bike bike
COPY petgraph petgraph

COPY src src

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --profile prod --recipe-path recipe.json --features prod

COPY gtfs-structure gtfs-structure
COPY gtfs-structure-2 gtfs-structure-2
COPY bike bike
COPY petgraph petgraph


RUN cargo build -p gtfs-structure-2 --profile prod

# Build application
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --profile prod --features prod

FROM base AS run
WORKDIR /app
#COPY certificates /app/certificates
#COPY city-gtfs /app/city-gtfs
#COPY web/public /app/web/public
COPY --from=builder /app/target/prod/timetoreach /usr/bin/timetoreach

ENV RUST_LOG info,timetoreach=debug,h2=info,hyper=info,warp=info,rustls=info
ENTRYPOINT ["/usr/bin/timetoreach"]