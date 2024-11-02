# Build stage
FROM rust:1.72 as builder

# Install protobuf compiler and musl-tools for static linking
RUN apt-get update && apt-get install -y protobuf-compiler musl-tools && rm -rf /var/lib/apt/lists/*

# Add the musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/app

# Copy the entire project
COPY . .

# Build the application using musl for static linking
RUN cargo build --release --target=x86_64-unknown-linux-musl

# Runtime stage - Use scratch for minimal size
FROM scratch

WORKDIR /app

# Copy the statically compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/grpc-client . 

# Set the startup command
CMD ["./grpc-client"]

# docker build -t grpc-client .