# This file is intended to be used in conjunction with `docker compose watch`.
# Caution: this file should only be used in development, **not** in production.

# ARGs only last for the build phase of a single image.
# For the multistage, renew the ARG by simply stating: ARG XXX
ARG APP_NAME=totp-server

################################################################################
FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx

################################################################################
# Create a stage for building the application.
FROM --platform=$BUILDPLATFORM rust:alpine AS build
ARG APP_NAME
WORKDIR /code

# Copy helper scripts from tonistiigi/xx
COPY --link --from=xx / /

# Install host build dependencies.
RUN apk add --no-cache musl-dev clang

# Fetch crates before building stage for better caching.
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    cargo fetch

# This is the architecture you're building for, which is passed in by the builder.
# Placing it here allows the previous steps to be cached across architectures.
# https://docs.docker.com/reference/dockerfile/#automatic-platform-args-in-the-global-scope
ARG TARGETPLATFORM
RUN mkdir -p /app/${TARGETPLATFORM}

# Copy all project files while respecting .dockerignore
COPY --link . .

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry for downloaded dependencies,
# a cache mount to /usr/local/cargo/git/db for git repository dependencies,
# and a cache mount to ./target/ for compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the source code into the container.
# Once built, copy the executable to an output directory before the cache mounted ./target is unmounted.
RUN --mount=type=cache,target=./target/,id=rust-cache-${APP_NAME}-${TARGETPLATFORM} \
    --mount=type=cache,target=/usr/local/cargo/registry,readonly \
    --mount=type=cache,target=/usr/local/cargo/git/db,readonly \
    xx-cargo build --offline --target-dir ./target && \
    xx-verify --static ./target/$(xx-cargo --print-target-triple)/debug/${APP_NAME} && \
    cp ./target/$(xx-cargo --print-target-triple)/debug/${APP_NAME} /app/${TARGETPLATFORM}

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base image
# from the build stage where the necessary files are copied from the build stage.

FROM busybox:musl AS final
WORKDIR /app
ARG APP_NAME

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/go/dockerfile-user-best-practices/
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Expose the port that the application listens on.
EXPOSE 7392

ARG TARGETPLATFORM
# Copy the executable from the "build" stage.
COPY --link --from=build /app/${TARGETPLATFORM}/${APP_NAME} .

# What the container should run when it is started.
CMD ["/app/totp-server"]
