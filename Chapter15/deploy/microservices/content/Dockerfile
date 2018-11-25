FROM rust:nightly

RUN USER=root cargo new --bin content-microservice
WORKDIR /content-microservice
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
RUN rm ./target/debug/deps/content_microservice*
RUN cargo build

CMD ["./target/debug/content-microservice"]

EXPOSE 8000
