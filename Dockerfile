# syntax=docker/dockerfile:1.4
FROM rust:1.77 as builder

WORKDIR /usr/src/

RUN <<EOF
apt-get update
apt-get install -y protobuf-compiler
apt-get clean
rm -rf /var/lib/apt/lists/*
EOF

ARG RUSTUP_TARGET="x86_64-unknown-linux-gnu"
ARG RUSTFLAGS="-C target-feature=+crt-static"

COPY . .

RUN --mount=type=cache,id=rustcache,target=/usr/local/cargo/registry --mount=type=cache,id=rustcache,target=./target <<EOF
# unwind not required
sed -i 's/^# panic/panic/' Cargo.toml

cargo build --release --target $RUSTUP_TARGET
EOF

RUN --mount=type=cache,id=rustcache,target=./target <<EOF
mv /usr/src/target/*/release/encelade-register-backend ./microservice
EOF

# Production image
FROM scratch

COPY --from=builder /usr/src/microservice /usr/local/bin/

CMD ["/usr/local/bin/microservice"]