ARG RUST_VERSION=1.68.2

FROM rust:${RUST_VERSION} as builder

WORKDIR /app
COPY . /app

RUN cargo build --release

FROM gcr.io/distroless/cc

USER nonroot

COPY --from=builder /app/target/release/magistr /

ENTRYPOINT ["/magistr"]
