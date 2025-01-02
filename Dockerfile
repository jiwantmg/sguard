# Stage 1: Build
FROM rust:1.79 AS builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the Rust project files into the container
COPY . .

# Build the Rust application in release mode
RUN cargo build --release

# Install GLIBC 2.33
WORKDIR /tmp
RUN apt-get update && apt-get install -y \
    wget bzip2 build-essential gawk bison \
    && wget http://ftp.gnu.org/gnu/libc/glibc-2.33.tar.gz && \
    tar -xvzf glibc-2.33.tar.gz && \
    cd glibc-2.33 && \
    mkdir build && cd build && \
    ../configure --prefix=/glibc && \
    make -j$(nproc) && \
    make install && \
    cd /tmp && rm -rf glibc-2.33*

# Stage 2: Runtime
FROM debian:bullseye-slim

# Set the working directory
WORKDIR /app

# Copy the compiled GLIBC and application binary from the builder stage
COPY --from=builder /glibc /glibc
COPY --from=builder /usr/src/app/target/release/sguard /app/sguard

# Update the library path to use the custom GLIBC
ENV LD_LIBRARY_PATH="/glibc/lib:$LD_LIBRARY_PATH"

# Expose application port (adjust as necessary)
EXPOSE 8000

# Command to run the application
CMD ["./sguard"]
