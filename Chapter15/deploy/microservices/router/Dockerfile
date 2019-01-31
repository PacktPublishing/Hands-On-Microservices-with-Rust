FROM rust:nightly

RUN USER=root cargo new --bin router-microservice
WORKDIR /router-microservice
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
COPY ./static ./static
RUN rm ./target/debug/deps/router_microservice*
RUN cargo build

CMD ["./target/debug/router-microservice"]

EXPOSE 8000
