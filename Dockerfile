FROM alpine:edge as builder

COPY src /source/src/
COPY Cargo.toml /source/
COPY Cargo.lock /source/

RUN apk add --no-cache --virtual .build-dependencies \
    cargo \
    build-base \
    file \
    libgcc \
    musl-dev \
    rust 
#RUN apk add --no-cache  openssl-dev
WORKDIR /source/
RUN cargo build --release




FROM alpine:edge
COPY config.toml /etc/poke-agent/config.toml
RUN apk add --no-cache llvm-libunwind libgcc 
COPY --from=builder /source/target/release/dns-agent /usr/bin/dns-agent
CMD ["/usr/bin/dns-agent", "--help"]