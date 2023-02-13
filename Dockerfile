FROM rust:latest AS builder

# Create project directory and copy files
WORKDIR /app
COPY . .

# Compile the program
RUN cargo build --release

# use google distroless as runtime image
FROM gcr.io/distroless/cc-debian11
WORKDIR /app
COPY --from=builder /app/target/release/djmxcreation-backend-axum /app/djmxcreation-backend-axum
COPY --from=builder /app/sql /app/sql

# Run the compiled program
CMD ["./djmxcreation-backend-axum"]