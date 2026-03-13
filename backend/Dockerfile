# Backend multi-stage build:
# 1. build with Rust
# 2. copy release binary into distroless final image

FROM quay.io/geoengine/devcontainer:latest AS builder

WORKDIR /usr/src/biois

# Cache dependencies: copy manifest first
COPY \
    Cargo.toml \
    Cargo.lock \
    rust-toolchain.toml \
    ./
# create dummy src to allow caching of cargo registry build step
# This is a best practice so that code changes don't invalidate the entire cargo build cache
RUN mkdir src && printf "fn main() { println!(\"cargo build cache\"); }\n" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy full source and build the release binary
COPY conf ./conf
COPY migrations ./migrations
COPY src ./src
COPY build.rs ./build.rs
RUN cargo build --release --bin biois

# Strip the binary to reduce size if possible
RUN strip target/release/biois || true

# Final image: distroless for minimal attack surface and size
FROM gcr.io/distroless/cc-debian13:nonroot

# Copy ca-certificates and the built binary
COPY --from=builder /usr/src/biois/target/release/biois /usr/local/bin/biois
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

USER nonroot
EXPOSE 4040
ENTRYPOINT ["/usr/local/bin/biois"]
