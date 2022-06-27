ARG BUILDER=registry.gitlab.com/gisleburt-homelab/rust-builder
ARG BASE=scratch

FROM $BUILDER as builder

ARG TARGET=aarch64-unknown-linux-musl

WORKDIR /home

COPY . .

RUN rustup target add $TARGET
RUN (cd status_lights_cli && cargo build --release --target=$TARGET)

FROM $BASE

ARG TARGET=aarch64-unknown-linux-musl

COPY --from=builder /home/status_lights_cli/target/$TARGET/release/status_lights /status_lights

ENTRYPOINT ["/status_lights"]
