FROM docker.io/library/alpine as builder
WORKDIR /build
RUN apk --no-cache upgrade && apk --no-cache add cargo

# cargo needs a dummy src/main.rs to detect bin mode
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# We need to touch our real main.rs file or the cached one will be used.
COPY . ./
RUN touch src/main.rs

RUN cargo build --release


# Start building the final image
FROM docker.io/library/alpine
VOLUME /app/eventfiles
VOLUME /app/additionalEventsGithub
WORKDIR /app

RUN apk --no-cache upgrade && apk --no-cache add libgcc bash git

COPY --from=builder /build/target/release/hawhh-calendarbot-downloader /usr/bin/

HEALTHCHECK --interval=5m \
    CMD bash -c '[[ $(find . -maxdepth 1 -name ".last-successful-run" -mmin "-250" -print | wc -l) == "1" ]]'

ENTRYPOINT ["hawhh-calendarbot-downloader"]
