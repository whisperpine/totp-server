<!-- markdownlint-disable MD033 -->
<h1 align="center">TOTP Server</h1>

<div align="center">

[![license](https://img.shields.io/badge/MIT_OR_Apache--2.0-blue?label=license)](https://github.com/whisperpine/totp-server/blob/main/LICENSE-APACHE)
[![checks](https://img.shields.io/github/actions/workflow/status/whisperpine/totp-server/checks.yml?logo=github&label=checks)](https://github.com/whisperpine/totp-server/actions/workflows/checks.yml)
[![build](https://img.shields.io/github/actions/workflow/status/whisperpine/totp-server/build.yml?logo=github&label=build)](https://github.com/whisperpine/totp-server/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/whisperpine/totp-server/graph/badge.svg?token=5PCNPENV26)](https://codecov.io/gh/whisperpine/totp-server)
[![release](https://img.shields.io/github/v/release/whisperpine/totp-server)](https://github.com/whisperpine/totp-server/releases)

Time-based One-time Password (TOTP) server.

</div>
<!-- markdownlint-enable MD033 -->

## Background

While developing a mobile game, I needed a way to include developer-only
features, such as a console for viewing logs, in the production app without
exposing them to regular users. Maintaining a separate "internal version" of the
app would have slowed down our iteration process, so I sought a lightweight and
secure solution. This led to the creation of totp-server.

With totp-server, developer-only features are unlocked only after successful
TOTP authentication. This approach eliminates the need to manage special
permissions at the user-identity level (e.g., adding a "superuser" flag in the
database). It's secure, cost-effective, and efficient, as developers can spin up
the TOTP server in under a minute via CI/CD for debugging and tear it down when
done.

## Authenticator

Assume that you have a TOTP client (e.g. Google Authenticator).

- When running locally (e.g. by `just run`), or deployed by the container image,
  find QR code in the logs.
- When running on AWS Lambda, the QR code isn't logged on AWS CloudWatch.
  In this case, entering a setup key in TOTP clients (e.g. Google Authenticator)
  is the only way. Get the setup key by the following command:

```sh
# replace `xxx` with `RAW_SECRET`
echo "xxx" | base32 | tr -d '='
```

## Deployment

### AWS Lambda

Any of the following approaches will deploy on AWS Lambda:

- Trigger the GitHub Actions workflow [infra-apply.yml](https://github.com/whisperpine/totp-server/actions/workflows/infra-apply.yml)
  (recommended).
- Run these command locally to deploy on AWS Lambda (*not* recommended):

```sh
cargo lambda build --release
cd ./infra && tofu apply
```

### Docker Compose

You can also deploy it by docker, for example:

```yaml
services:
  totp-server:
    init: true # Terminate container immediately when pressing `ctrl-c`.
    image: ghcr.io/whisperpine/totp-server
    restart: unless-stopped
    ports: ["9000:9000"]
    environment:
      RUST_LOG: totp_server=info # trace, debug, info (default), warn, error.
      RAW_SECRET: "xxx" # Required: It should be at least 16 chars. 
      TCP_BIND_PORT: 9000 # Optional: TCP port (default: 9000).
      REQUEST_RATE_LIMIT: 25 # Optional: Rate limit per 30 seconds (default: 25).
```

## Dev Environment

Nix flake and and [direnv](https://github.com/direnv/direnv)
is used in this project to handle dev environment.
Although this solution is recommended, it's not mandatory.
Instead you can manually install tools declared
in `with pkgs; [ ]` of [flake.nix](./flake.nix) file.

Boot the dev server locally that emulates AWS Lambda:

```sh
# Boot the dev server that emulates interactions with the AWS Lambda.
just watch
# Send http request defined in a hurl file.
# "Error: invalid TOTP" will occur unless the "token" field is set correctly.
hurl ./hurl/totp.hurl
```
