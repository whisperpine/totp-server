# TOTP Server

[![license](https://img.shields.io/badge/MIT_OR_Apache--2.0-blue?label=license)](https://github.com/whisperpine/totp-server/blob/main/LICENSE-APACHE)
[![checks](https://img.shields.io/github/actions/workflow/status/whisperpine/totp-server/checks.yml?logo=github&label=checks)](https://github.com/whisperpine/totp-server/actions/workflows/checks.yml)
[![build](https://img.shields.io/github/actions/workflow/status/whisperpine/totp-server/build.yml?logo=github&label=build)](https://github.com/whisperpine/totp-server/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/whisperpine/totp-server/graph/badge.svg?token=5PCNPENV26)](https://codecov.io/gh/whisperpine/totp-server)
[![release](https://img.shields.io/github/v/release/whisperpine/totp-server)](https://github.com/whisperpine/totp-server/releases)

Time-based One-time Password (TOTP) web server.

## Authenticator

QRCode is much more user-friendly, which should be implemented later on.
At the moment, enter a setup key in TOTP clients (e.g. Google Authenticator)
is the only way to add a record.

Get the setup key by the following command:

```sh
# replace `xxx` with `RAW_SECRET`
echo "xxx" | base32 | tr -d '='
```

## Deployment

### Docker Compose

```yaml
services:
  totp-server:
    init: true # Terminate container immediately when pressing `ctrl-c`.
    image: ghcr.io/whisperpine/totp-server
    restart: unless-stopped
    ports:
      - 7392:7392
    environment:
      # Log level candidates: trace, debug, info, warn, error.
      RUST_LOG: totp_server=debug
      # Set RAW_SECRET in .env file. It should be at least 16 chars.
      RAW_SECRET: ${RAW_SECRET}
      # TCP port. Default: 7392.
      TCP_BIND_PORT: 7392
      # Request rate limit in 30 seconds. Default: 25.
      REQUEST_RATE_LIMIT: 25
```

## Dev Environment

Nix flake and and [direnv](https://github.com/direnv/direnv)
is used in this project to handle dev environment.
Although this solution is recommended, it's not mandatory.
Instead you can manually install tools declared
in `with pkgs; [ ]` of [flake.nix](./flake.nix) file.

### Configurations

Env var `RAW_SECRET` should be set in the `.env` file.
`RAW_SECRET` must be composed of at least 16 characters.
It's optional to set `RAW_SECRET` in local dev environment,
but is mandatory to set when deployed.

Refer to the [Docker Compose](#docker-compose) section to see the full picture
of environment variables.

### Run and Test

[hurl](https://github.com/Orange-OpenSource/hurl)
is used in local test, so it must be installed beforehand.

```sh
# Run totp-server in the log level of debug.
just run
# Send http request defined in a hurl file.
# "Error: invalid TOTP" will occur unless the "token" field is set correctly.
hurl ./hurl/totp.hurl
```
