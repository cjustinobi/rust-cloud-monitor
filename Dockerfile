FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Match the build environment
FROM rust:latest
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust-cloud-monitor /usr/local/bin/monitor
COPY config.yaml /app/config.yaml
WORKDIR /app
EXPOSE 8080
CMD ["monitor"]

# FROM rust:latest as builder
# WORKDIR /app
# COPY . .
# RUN cargo build --release

# FROM debian:bookworm-slim
# RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /app/target/release/rust-cloud-monitor /usr/local/bin/monitor
# COPY config.yaml /app/config.yaml
# WORKDIR /app
# EXPOSE 8080
# CMD ["monitor"]