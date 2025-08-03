output "lambda_function_url" {
  description = "the url of the aws lambda function"
  value       = aws_lambda_function_url.default.function_url
}
