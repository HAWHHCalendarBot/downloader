FROM ekidd/rust-musl-builder as builder

WORKDIR /home/rust

# cargo needs a dummy src/main.rs to detect bin mode
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

COPY Cargo.toml Cargo.lock ./
RUN cargo test
RUN cargo build --release

# We need to touch our real main.rs file or else docker will use
# the cached one.
COPY . ./
RUN sudo touch src/main.rs

RUN cargo test
RUN cargo build --release

# Size optimization
RUN strip target/x86_64-unknown-linux-musl/release/downloader


# Start building the final image
FROM alpine
VOLUME /app/eventfiles
VOLUME /app/additionalEventsGithub
WORKDIR /app

RUN apk --no-cache add bash git

COPY --from=builder /home/rust/target/x86_64-unknown-linux-musl/release/downloader /usr/bin/

HEALTHCHECK --interval=5m \
    CMD bash -c '[[ $(find . -maxdepth 1 -name ".last-successful-run" -mmin "-250" -print | wc -l) == "1" ]]'

ENTRYPOINT ["downloader"]
