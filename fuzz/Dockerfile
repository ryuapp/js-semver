FROM rust:bookworm

ARG CARGO_FUZZ_VERSION=0.13.2
ARG CARGO_FUZZ_SHA256=b5b704018b63e0f151c17a057ac53b5111e1db545d1b9f72fee79f08a545931c

RUN rustup toolchain install nightly --profile minimal
RUN curl --fail --location --silent --show-error \
    "https://github.com/rust-fuzz/cargo-fuzz/releases/download/${CARGO_FUZZ_VERSION}/cargo-fuzz-${CARGO_FUZZ_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    --output /tmp/cargo-fuzz.tar.gz \
    && echo "${CARGO_FUZZ_SHA256}  /tmp/cargo-fuzz.tar.gz" | sha256sum --check --strict \
    && tar --extract --gzip --file /tmp/cargo-fuzz.tar.gz --directory /usr/local/bin \
    && rm /tmp/cargo-fuzz.tar.gz

WORKDIR /workspace

ENTRYPOINT ["cargo", "+nightly"]
