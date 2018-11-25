FROM rust:nightly

RUN USER=root cargo new --bin users-microservice
WORKDIR /users-microservice
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
COPY ./diesel.toml ./diesel.toml
RUN rm ./target/debug/deps/users_microservice*
RUN cargo build

CMD ["./target/debug/users-microservice"]

EXPOSE 8000
