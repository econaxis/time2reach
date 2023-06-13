FROM ubuntu:20.04

WORKDIR /app

COPY deploy/init.sh deploy/
RUN sh deploy/init.sh


ENV DEBIAN_FRONTEND=noninteractive
COPY deploy/deploy.sh deploy/
RUN sh deploy/deploy.sh
ENV PATH="/root/.cargo/bin:${PATH}"



COPY . .
RUN cargo build --release


ENTRYPOINT ["cargo", "run", "--release"]