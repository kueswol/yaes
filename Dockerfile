FROM rust:1-slim AS builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /usr/src/app/target/release/yaes /app/
EXPOSE 3000
CMD ["/app/yaes"]
