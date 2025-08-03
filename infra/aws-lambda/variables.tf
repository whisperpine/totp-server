variable "function_name" {
  default = "aws lambda function name"
  type    = string
}

variable "zip_file_path" {
  description = "The relative file path of bootstrap.zip, built by cargo lambda"
  type        = string
}


variable "lambda_env_var" {
  description = "the environment variable for aws lambda function"
  type        = map(string)
  sensitive   = true
}
