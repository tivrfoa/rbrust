FROM rust:1.71.1

RUN apt-get update -yqq && apt-get install -yqq cmake g++

WORKDIR /actix

RUN mkdir src; touch src/main.rs

COPY Cargo.toml Cargo.lock ./

RUN cargo fetch

COPY src/ ./src/

RUN cargo build --release

EXPOSE 80

# ENV RUST_BACKTRACE=FULL

CMD ./target/release/rinha-backend-rust
