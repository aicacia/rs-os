FROM rust:1.91-trixie AS chef

RUN apt update && apt -yq upgrade
RUN apt -yq install musl-tools libpq-dev

WORKDIR /app

RUN rustup default stable

ARG TARGET=x86_64-unknown-linux-musl
RUN rustup target add ${TARGET}

RUN cargo install cargo-chef --locked

COPY . .

RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS planner

WORKDIR /app

RUN cargo chef cook --release --recipe-path recipe.json


FROM chef AS builder

WORKDIR /app

COPY --from=planner /app/target /app/target
COPY --from=planner /usr/local/cargo /usr/local/cargo
COPY . .

RUN cargo build -p os --target ${TARGET} --release


FROM scratch
LABEL org.opencontainers.image.source=https://github.com/aicacia/rs-os

WORKDIR /app

ARG TARGET=x86_64-unknown-linux-musl
COPY --from=builder /app/target/${TARGET}/release/os /app/os

ENV RUN_MODE=production

CMD ["/app/os", "-c", "/app/config.json"]
