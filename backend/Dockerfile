# Backend multi-stage build:
# 1. build with Rust
# 2. copy release binary into distroless final image

FROM quay.io/geoengine/devcontainer:latest AS builder

WORKDIR /usr/src/biois

# Cache dependencies: copy manifest first
COPY \
    build.rs \
    Cargo.toml \
    Cargo.lock \
    rust-toolchain.toml \
    Toasty.toml \
    ./
# create dummy src to allow caching of cargo registry build step
# This is a best practice so that code changes don't invalidate the entire cargo build cache
RUN mkdir -p src/bin && \
    printf "fn main() { eprintln!(\"cargo build cache\"); }\n" > src/bin/server.rs && \
    printf "fn main() { eprintln!(\"cargo build cache\"); }\n" > src/bin/cli.rs
RUN mkdir -p macros/src && touch macros/src/lib.rs
COPY macros/Cargo.toml macros/Cargo.toml
RUN cargo build --release
RUN rm -rf \
    src \
    macros \
    target/release/deps/libbiois_macros-*

# Copy full source and build the release binary
COPY conf ./conf
COPY database ./database
COPY macros ./macros
COPY src ./src
COPY Toasty.toml ./
RUN cargo build --release --bin biois-server

# Strip the binary to reduce size if possible
RUN strip target/release/biois-server || true

# Create an empty directory to use as workdir in the final image
RUN mkdir /emptydir

# Final image: distroless for minimal attack surface and size
FROM gcr.io/distroless/cc-debian13:nonroot

# Copy ca-certificates and the built binary
COPY --from=builder /usr/src/biois/target/release/biois-server /usr/local/bin/biois-server
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /emptydir /biois

WORKDIR /biois

# ENV PUBLIC_URL="https://biois.app.geoengine.io/api"

USER nonroot
EXPOSE 4040
ENTRYPOINT ["/usr/local/bin/biois-server"]
