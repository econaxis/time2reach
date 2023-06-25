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
COPY src src

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --features https,all-cities

FROM base AS run
WORKDIR /app
COPY --from=builder /app/city-gtfs /app/city-gtfs
COPY --from=builder /app/certificates /app/certificates
COPY --from=builder /app/web/public /app/web/public
COPY --from=builder /app/target/release/timetoreach /app/target/release/timetoreach


ENTRYPOINT ["/app/target/release/timetoreach"]