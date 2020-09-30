FROM rust:latest

RUN apt-get update && apt-get install nano

RUN mkdir -p /app/src
WORKDIR /app/src

COPY src .

WORKDIR /app
COPY Cargo.toml Cargo.lock ./

