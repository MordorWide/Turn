# Use the official Rust image for building the application
FROM rust:1.83-slim-bookworm AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo manifest and lock file first to leverage Docker caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the actual application
RUN cargo build --release

# Use a lightweight image for running the application
FROM debian:bookworm-slim

# Set the working directory for the runtime container
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/MordorWide-UDPTurn /app/MordorWide-UDPTurn

# Set the default command for the container
CMD ["/app/MordorWide-UDPTurn"]
