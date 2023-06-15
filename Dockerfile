FROM ubuntu:20.04 AS chef

WORKDIR /app

COPY deploy/init.sh deploy/
RUN sh deploy/init.sh


ENV DEBIAN_FRONTEND=noninteractive
COPY deploy/deploy.sh deploy/
RUN sh deploy/deploy.sh
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install cargo-chef

FROM chef as planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

FROM ubuntu:20.04 AS run
WORKDIR /app
COPY --from=builder /app/ /app/

ENTRYPOINT ["/app/target/release/timetoreach"]