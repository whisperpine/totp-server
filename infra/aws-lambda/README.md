# AWS Lambda Module

Create an AWS Lambda function and the corresponding CloudWatch Log Group.

## Prerequisites

- An AWS IAM user with permissions to manage AWS Lambda, CloudWatch Logs and IAM
  roles (e.g. predefined policy `AWSLambda_FullAccess`,
  `CloudWatchLogsFullAccess` and `IAMFullAccess`).
- The compiled executable compressed in a zip file (refer to `zip_file_path` in [./variables.tf](./variables.tf)).
  In this repository, run `cargo lambda build --release` or `just build` to get
  the zip file.
