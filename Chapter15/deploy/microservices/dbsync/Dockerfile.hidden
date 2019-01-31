FROM rust:nightly as builder

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

FROM buildpack-deps:stretch

COPY --from=builder /dbsync-worker/target/debug/dbsync-worker  /app/

ENV RUST_LOG=debug

ENTRYPOINT ["/app/dbsync-worker"]

EXPOSE 8000

