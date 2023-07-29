FROM --platform=$BUILDPLATFORM rust:alpine AS build

LABEL authors="Richard Stöckl"

RUN apk update
RUN apk add gcc-aarch64-none-elf clang16 llvm16 musl-dev
RUN rustup target add aarch64-unknown-linux-musl

ARG TARGETARCH
ARG CC_aarch64_unknown_linux_musl=clang
ARG AR_aarch64_unknown_linux_musl=llvm-ar
ARG CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"

COPY Cargo.toml Cargo.lock scripts/docker-multiarch-build.sh /workspace/
COPY src/ /workspace/src

WORKDIR /workspace

RUN ls /workspace

RUN --mount=type=cache,target=/workspace/target \
    sh /workspace/docker-multiarch-build.sh

FROM --platform=$TARGETPLATFORM alpine

RUN apk update && \
    apk --no-cache add ca-certificates && \
    rm -rf /var/cache/apk

ARG TARGETARCH
COPY --from=build /target/release/openkeg.$TARGETARCH /bin/openkeg

ENV RUST_BACKTRACE 1
ENV RUST_LOG info
ENV RUST_LOG_STYLE always

RUN mkdir /data
WORKDIR /data
VOLUME /data

EXPOSE 1926

CMD ["/bin/openkeg"]