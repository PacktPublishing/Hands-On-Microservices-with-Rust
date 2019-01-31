FROM rust:nightly

RUN USER=root cargo new --bin mails-microservice
WORKDIR /mails-microservice
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build

RUN rm src/*.rs
COPY ./src ./src
COPY ./templates ./templates
RUN rm ./target/debug/deps/mails_microservice*
RUN cargo build

CMD ["./target/debug/mails-microservice"]
