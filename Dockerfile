FROM ubuntu:20.04

WORKDIR /app

COPY deploy/init.sh deploy/
RUN sh deploy/init.sh


ENV DEBIAN_FRONTEND=noninteractive
COPY deploy/deploy.sh deploy/
RUN sh deploy/deploy.sh
ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get install -y sqlite3 cmake libproj-dev proj-bin
RUN apt-get install -y libclang-dev

COPY . .
RUN cargo build --release


ENTRYPOINT ["cargo", "run", "--release"]