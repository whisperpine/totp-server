terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.30.0"
    }
  }
}

# ------- #
# AWS IAM
# ------- #

# IAM Role for Lambda.
# https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/iam_role
resource "aws_iam_role" "lambda_role" {
  name = "totp-server-lambda-iam"
  # Policy that grants an entity permission to assume the role.
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "lambda.amazonaws.com"
      }
    }]
  })
}

# Attach IAM role with an AWS managed policy "AWSLambdaBasicExecutionRole".
# https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/iam_role_policy_attachment
resource "aws_iam_role_policy_attachment" "lambda_basic_execution" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# --------------- #
# Lambda Function
# --------------- #

# https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/lambda_function
resource "aws_lambda_function" "default" {
  function_name = var.function_name
  role          = aws_iam_role.lambda_role.arn
  handler       = "bootstrap"       # Function entry point in your code. Required if package_type is Zip.
  runtime       = "provided.al2023" # Identifier of the function's runtime. Required if package_type is Zip.
  architectures = ["arm64"]         # Graviton support for better price/performance.
  memory_size   = 128               # Valid value between 128 MB to 10,240 MB (10 GB), in 1 MB increments.
  timeout       = 1                 # Amount of time your Lambda Function has to run in seconds.
  # Path to the function's deployment package within the local filesystem.
  # One of "filename", "image_uri", or "s3_bucket" must be specified.
  filename = var.zip_file_path
  # Base64-encoded SHA256 hash of the package file. Used to trigger updates when source code changes.
  source_code_hash = filebase64sha256(var.zip_file_path)
  # Environment variables.
  environment {
    variables = var.lambda_env_var
  }
  # Ensure Log Group is created before Lambda if logging is enabled.
  depends_on = [aws_cloudwatch_log_group.default]
}

# ----------------- #
# Public Invocation
# ----------------- #

# Allows public invocation of the function.
# https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/lambda_permission
resource "aws_lambda_permission" "allow_public_invocation" {
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.default.function_name
  principal     = "*" # anyone (public)
}

# Allows public invocation of the function via its Function URL.
# https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/lambda_permission
resource "aws_lambda_permission" "allow_public_function_url" {
  action                 = "lambda:InvokeFunctionUrl"
  function_name          = aws_lambda_function.default.function_name
  principal              = "*"    # anyone (public)
  function_url_auth_type = "NONE" # must match your function URL auth type
}

# Lambda Function URL.
# https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/lambda_function_url
resource "aws_lambda_function_url" "default" {
  function_name      = aws_lambda_function.default.function_name
  authorization_type = "NONE" # public access
}

# ---------- #
# CloudWatch
# ---------- #

# CloudWatch Log Group.
# https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/cloudwatch_log_group
resource "aws_cloudwatch_log_group" "default" {
  name              = "/aws/lambda/${var.function_name}"
  retention_in_days = var.log_retention_days
  lifecycle {
    create_before_destroy = true
  }
}
