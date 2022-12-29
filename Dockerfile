FROM lukemathwalker/cargo-chef:latest-rust-1.66 as chef
WORKDIR /app

FROM chef AS planner
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build  --release --bin rust2prod 




FROM debian:bullseye-slim AS runtime 
WORKDIR /app
RUN apt update -y \
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust2prod rust2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production


ENTRYPOINT ["./rust2prod"]