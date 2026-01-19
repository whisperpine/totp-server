# Provider "carlpett/sops" docs:
# https://registry.terraform.io/providers/carlpett/sops/latest/docs
provider "sops" {}

# Provider "hashicorp/aws" docs:
# https://registry.terraform.io/providers/hashicorp/aws/latest/docs
provider "aws" {
  region     = local.aws_provider_region
  access_key = local.aws_access_key_id
  secret_key = local.aws_secret_access_key
  default_tags {
    tags = local.default_tags
  }
}

# Test the root module.
run "test_root_module" {
  command = plan
}
