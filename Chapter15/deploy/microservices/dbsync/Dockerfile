FROM rust:nightly

RUN USER=root cargo new --bin dbsync-worker
WORKDIR /dbsync-worker
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
COPY ./migrations ./migrations
COPY ./diesel.toml ./diesel.toml
RUN rm ./target/debug/deps/dbsync_worker*
RUN cargo build

CMD ["./target/debug/dbsync-worker"]

EXPOSE 8000

