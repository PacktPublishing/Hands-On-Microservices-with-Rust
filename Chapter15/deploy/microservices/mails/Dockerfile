FROM rust:1.30.1

WORKDIR /microservice
COPY . .

RUN cargo install --path /microservice

CMD ["mails-microservice"]
