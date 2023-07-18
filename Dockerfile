FROM rust:1.70 AS builder
RUN apt-get update && apt-get install -y protobuf-compiler libclang-dev
WORKDIR /app
COPY . .
RUN cargo build --release

FROM postgres:15.3
WORKDIR /app
COPY --from=builder /app/target/release/delivery_user .
CMD ["/app/delivery_user"]