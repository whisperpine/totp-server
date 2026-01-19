# ------------------ #
# module: aws_lambda
# ------------------ #

output "lambda_function_url" {
  description = "the url of the aws lambda function"
  value       = module.aws_lambda.lambda_function_url
  sensitive   = true
}

output "lambda_env_var" {
  description = "the environment variable of the aws lambda function"
  value       = local.lambda_env_var
  sensitive   = true
}
