FROM rust:1.61-alpine3.15 as builder

RUN apk add --no-cache \
        musl-dev

WORKDIR /app

RUN mkdir ./src
COPY Cargo.toml /app/
COPY Cargo.lock /app/
COPY docker/dummy.rs ./src/main.rs

RUN cargo build --release  # to fetch dependencies only and cache them

RUN rm -rf /app/src
COPY src/ src/
RUN touch src/main.rs # prevent cache

RUN cargo build --bins --release

FROM alpine:3.14
COPY --from=builder /app/target/release/server /server
EXPOSE 31337
ENTRYPOINT ["/server"]
