FROM rust:1.92-trixie AS chef

RUN apt update && apt -yq upgrade
RUN apt -yq install musl-tools libpq-dev

WORKDIR /app

RUN rustup default stable

ARG TARGET=x86_64-unknown-linux-musl

RUN rustup target add ${TARGET}

RUN cargo install cargo-chef --locked


FROM chef AS planner

WORKDIR /app

COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

ARG TARGET=x86_64-unknown-linux-musl
ARG PROJECT=os

COPY . .
RUN cargo build -p ${PROJECT} --target ${TARGET} --release --bin ${PROJECT}


FROM scratch
LABEL org.opencontainers.image.source=https://github.com/aicacia/rs-os

WORKDIR /app

ARG TARGET=x86_64-unknown-linux-musl
ARG PROJECT=os

COPY --from=builder /app/target/${TARGET}/release/${PROJECT} /app/run

CMD ["/app/run", "-c", "/app/config.json"]