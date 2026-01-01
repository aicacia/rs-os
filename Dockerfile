FROM rust:1.92-trixie AS chef

RUN apt update && apt -yq upgrade
RUN apt -yq install musl-tools libpq-dev

WORKDIR /app

RUN rustup default stable

ARG TARGETPLATFORM=linux/amd64
RUN case "${TARGETPLATFORM}" in \
  linux/amd64) echo "x86_64-unknown-linux-musl" > /tmp/target ;; \
  linux/arm64) echo "aarch64-unknown-linux-musl" > /tmp/target ;; \
  linux/arm/v7) echo "armv7-unknown-linux-musleabihf" > /tmp/target ;; \
  linux/arm/v6) echo "arm-unknown-linux-musleabi" > /tmp/target ;; \
  linux/386) echo "i686-unknown-linux-musl" > /tmp/target ;; \
  linux/riscv64) echo "riscv64gc-unknown-linux-musl" > /tmp/target ;; \
  linux/ppc64le) echo "powerpc64le-unknown-linux-musl" > /tmp/target ;; \
  linux/s390x) echo "s390x-unknown-linux-musl" > /tmp/target ;; \
  *) echo "x86_64-unknown-linux-musl" > /tmp/target ;; \
  esac

RUN rustup target add $(cat /tmp/target)

RUN cargo install cargo-chef --locked


FROM chef AS planner

WORKDIR /app

COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

ARG PROJECT=os

COPY . .
RUN cargo build -p ${PROJECT} --target $(cat /tmp/target) --release --bin ${PROJECT}


FROM scratch
LABEL org.opencontainers.image.source=https://github.com/aicacia/rs-os

WORKDIR /app

ARG PROJECT=os

COPY --from=builder /app/target/*/release/${PROJECT} /app/run

CMD ["/app/run", "-c", "/app/config.json"]