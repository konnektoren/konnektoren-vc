FROM rust:bookworm as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && touch src/lib.rs
RUN cargo check
COPY src ./src
COPY assets ./assets
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt install -y openssl ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY assets ./assets
COPY --from=builder /app/target/release/konnektoren-vc ./konnektoren-vc

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./konnektoren-vc"]
