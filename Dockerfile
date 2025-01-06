# Stage 1: Build
FROM rust:1.79 AS builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the entire project into the container
COPY . .

# Build the Rust application in release mode
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Set the working directory
WORKDIR /app

# Install necessary runtime libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/sguard /app/sguard

COPY --from=builder /usr/src/app/log4rs.yaml /app/log4rs.yaml
COPY --from=builder /usr/src/app/routes.yaml /app/routes.yaml

# Expose application port (adjust as necessary)
EXPOSE 8080

# Command to run the application
CMD ["./sguard"]