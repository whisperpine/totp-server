# https://registry.terraform.io/providers/carlpett/sops/latest/docs/data-sources/file
data "sops_file" "default" {
  source_file = "encrypted.${terraform.workspace}.json"
}

locals {
  # Tags for hashicorp/aws provider.
  repository = "totp-server"
  default_tags = {
    tf-workspace  = terraform.workspace
    tf-repository = local.repository
  }
  # Provider: hashicorp/aws.
  aws_provider_region   = data.sops_file.default.data["aws_provider_region"]
  aws_access_key_id     = data.sops_file.default.data["aws_access_key_id"]
  aws_secret_access_key = data.sops_file.default.data["aws_secret_access_key"]
  # Module: aws-lambda.
  totp_server_raw_secret = data.sops_file.default.data["totp_server_raw_secret"]
  lambda_env_var = {
    RUST_LOG   = "totp_server=info"
    RAW_SECRET = local.totp_server_raw_secret
  }
}

# Create commonly used aws resources (e.g. aws resource group).
module "aws_common" {
  source                  = "./aws-common"
  aws_resource_group_name = "${local.repository}-${terraform.workspace}"
  default_tags            = local.default_tags
}

# Create an AWS Lambda function and the corresponding CloudWatch Log Group.
module "aws_lambda" {
  source             = "./aws-lambda"
  function_name      = "totp-server-${terraform.workspace}"
  zip_file_path      = "../target/lambda/totp-server/bootstrap.zip"
  lambda_env_var     = local.lambda_env_var
  log_retention_days = 7
}
