ARG RUST_VERSION=1.89.0

FROM rust:${RUST_VERSION} AS builder

WORKDIR /app
COPY . /app

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

USER nonroot

COPY --from=builder /app/target/release/magistr /

ENTRYPOINT ["/magistr"]
