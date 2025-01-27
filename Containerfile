FROM rust

RUN apt update && apt install cmake clang -y

WORKDIR /build

COPY . .
RUN cargo build --release