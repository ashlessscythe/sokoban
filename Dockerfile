# Start with a Rust image to build the project
FROM rust:bullseye as builder

# Create a new empty shell project
RUN USER=root cargo new --bin sokoban
WORKDIR /sokoban

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# Now that the dependencies are built, copy your source code
COPY ./src ./src
COPY ./migrations ./migrations
COPY ./templates ./templates
COPY ./static ./static
COPY ./Rocket.toml ./Rocket.toml
# If you use dotenv
COPY ./.env ./.env

# Build for release.
RUN rm ./target/release/deps/sokoban*
RUN cargo build --release

# The final stage
# Start from a fresh image to reduce the size
FROM debian:bullseye-slim

# Install needed packages including OpenSSL
RUN apt-get update && apt-get install -y libpq5 openssl && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage.
COPY --from=builder /sokoban/target/release/sokoban .

# Set the default command to run when starting the container
CMD ["./sokoban"]
