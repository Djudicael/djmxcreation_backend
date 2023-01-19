FROM rust:latest as builder


# Create a new project directory and copy the project files
RUN mkdir -p /usr/src/djmxcreation-backend-axum
WORKDIR /usr/src/djmxcreation-backend-axum
COPY . .


# Compile the program
RUN cargo build --release
# RUN cargo build -p djmxcreation-backend-axum --release


# copy the binary to a smaller image
FROM alpine
COPY --from=builder /usr/src/djmxcreation-backend-axum/target/release/djmxcreation-backend-axum /usr/local/bin/
COPY --from=builder /usr/src/djmxcreation-backend-axum/sql /usr/local/bin/

# Run the compiled program
CMD ["djmxcreation-backend-axum"]