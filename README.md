# README

Time-based One-time Password (TOTP) web server.

## Development

## Dev Environment

Nix flake is used in this project to handle dev environment,
in conjunction with [direnv](https://github.com/direnv/direnv) and [nix-direnv](https://github.com/nix-community/nix-direnv).
Although this solution is recommended, it's not mandatory.
Instead you can manually install tools declared
in `with pkgs; [ ]` of [flake.nix](./flake.nix) file.

### Configurations

Environment variable `RAW_SECRET` should be set in the `.env` file.
`RAW_SECRET` must be composed of at least 16 characters.
It's recommended to set `RAW_SECRET` in local dev environment,
and it must be set in any kind of deployment.

Refer to the [Docker Compose](#docker-compose) section to see the full picture
of environment variables.

### Run and Test

[hurl](https://github.com/Orange-OpenSource/hurl)
is used in local test, so it must be installed beforehand.

```sh
# Run totp-server in the log level of debug.
RUST_LOG="totp_server=debug" cargo run
# Send http request defined in a hurl file.
# "Error: invalid TOTP" will occur unless the "token" field is set correctly.
hurl ./hurl/totp.hurl
```

### Docker Compose Watch

This approach is closer to deployment, but is a little bit hassled.

```sh
# Watch Build context for service and rebuild/refresh containers when files are updated.
docker compose watch
```

Configurations can be found in the `build` field of [compose.yaml](./compose.yaml).

## Deployment

### Build the Image

```sh
# Build the container image.
docker build -t totp-server .

# Build the multi-platform container image.
docker build --platform linux/amd64,linux/arm64 -t totp-server .
```

### Docker Compose

```yaml
services:
  totp-server:
    init: true # Terminate container immediately when pressing `ctrl-c`.
    image: whisperpine/totp-server
    restart: unless-stopped
    environment:
      # Log level candidates: trace, debug, info, warn, error.
      RUST_LOG: totp_server=debug
      # Set RAW_SECRET in .env file. It should be at least 16 chars.
      RAW_SECRET: ${RAW_SECRET}
      # TCP port. Default: 7392.
      TPC_BIND_PORT: 7392
      # Request rate limit in 30 seconds. Default: 25.
      REQUEST_RATE_LIMIT: 25
```

## Authenticator

QRCode is much more user-friendly, which should be implemented later on.
At the moment, enter a setup key in TOTP clients (e.g. Google Authenticator)
is the only way to add a record.

Get the setup key by the following command:

```sh
# replace `xxx` with `RAW_SECRET`
echo "xxx" | base32 | tr -d '='
```
