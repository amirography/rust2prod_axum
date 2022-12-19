FROM rust:1.66.0

WORKDIR /app

COPY . .

RUN cargo build --release

ENTRYPOINT ["./target/release/rust2prod"]