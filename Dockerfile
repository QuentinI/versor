FROM rust:1.48
WORKDIR /usr/src
 
RUN USER=root apt install pkg-config openssl
 
RUN USER=root cargo new versor
WORKDIR /usr/src/versor
COPY Cargo.toml Cargo.lock ./
# Cache dependencies
RUN cargo build --release
 
COPY src ./src
# Build the actual executable
RUN cargo install --path . --root /usr
 
WORKDIR /versor
CMD ["/usr/bin/versor"]
