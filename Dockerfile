FROM rust:latest as builder


# Create a new project directory and copy the project files
RUN mkdir -p /usr/src/djmxcreation-backend-axum
WORKDIR /usr/src/djmxcreation-backend-axum
COPY . /usr/src/djmxcreation-backend-axum


# Compile the program
RUN cargo build -p djmxcreation-backend-axum --release
# RUN cargo build -p djmxcreation-backend-axum --release


# copy the binary to a smaller image
FROM alpine:latest
RUN mkdir -p ./app


COPY --from=builder /usr/src/djmxcreation-backend-axum/target/release/djmxcreation-backend-axum ./app

COPY --from=builder /usr/src/djmxcreation-backend-axum/sql ./


# Run the compiled program
CMD ["./app/djmxcreation-backend-axum"]