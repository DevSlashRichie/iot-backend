FROM public.ecr.aws/docker/library/rust:1-slim-bullseye AS builder

WORKDIR /usr/app

RUN mkdir -p /usr/crates

COPY . . 

COPY ./crates/ /usr/crates

RUN cargo build --release

FROM public.ecr.aws/docker/library/debian:bookworm-slim
WORKDIR /usr/app

LABEL org.opencontainers.image.source="https://github.com/DevSlashRichie/iot-backend"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl

RUN update-ca-certificates

COPY --from=builder /usr/app/target/release/iot-cli ./iot-cli

ENTRYPOINT ["./iot-cli"]
