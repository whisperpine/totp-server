variable "function_name" {
  default = "aws lambda function name"
  type    = string
}

variable "zip_file_path" {
  description = "The relative file path of bootstrap.zip, built by cargo lambda"
  type        = string
  validation {
    condition     = fileexists(var.zip_file_path)
    error_message = "no such file in ${var.zip_file_path}"
  }
}

variable "lambda_env_var" {
  description = "the environment variable for aws lambda function"
  type        = map(string)
  sensitive   = true
}

variable "log_retention_days" {
  description = "the retention days of logs in AWS Cloudwatch"
  type        = number
  validation {
    condition     = var.log_retention_days >= 1 && var.log_retention_days <= 180
    error_message = "the log retention days should be ranged from 1 to 180"
  }
}
