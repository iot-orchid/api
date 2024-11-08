# Use the official Rust image as the base image
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

# Copy the source code to the container
COPY src ./src
COPY entity ./entity
COPY migration ./migration

# Build the application
RUN cargo build --release

# Create a new stage with a smaller base image
FROM debian:buster-slim

# Set the working directory inside the container
WORKDIR /app

# Copy the built binary from the previous stage
COPY --from=builder /app/target/release/api .

# Expose the port that your program listens on (if applicable)
EXPOSE 8080

# Set the command to run your program
CMD ["./iot-orchid-api"]