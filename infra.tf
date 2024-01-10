terraform {

  backend "s3" {
    bucket = "my-sites-terraform-remote-state"
on the previous deployment, terraform failed to push state to S3. this is a breaking reminder to address that locally before deploying from CI again 
    key    = "calcount-state"
    region = "us-east-2"
  }

  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = ">= 2.7.1"
    }
    helm = {
      source  = "hashicorp/helm"
      version = ">= 2.4.1"
    }
  }
}

provider "kubernetes" {
  config_path = "~/.kube/config"
}

provider "helm" {
  kubernetes {
    config_path = "~/.kube/config"
  }
}

variable "openai_api_key" {
  type      = string
  sensitive = true
}

variable "stripe_api_key" {
  type      = string
  sensitive = true
}

variable "stripe_webhook_signing_secret" {
  type      = string
  sensitive = true
}

variable "smtp_email_password" {
  type      = string
  sensitive = true
}

resource "random_password" "secret_key" {
  length  = 48
  special = false
}

data "external" "git_describe" {
  program = [
    "sh",
    "-c",
    "echo '{\"output\": \"'\"$(if [[ ! -z $GITLAB_SHA ]]; then echo $GITLAB_SHA; else git rev-parse HEAD; fi)\"'\"}'"
  ]
}

module "basic-deployment" {
  source  = "jdevries3133/basic-deployment/kubernetes"
  version = "3.0.2"

  app_name  = "calcount"
  container = "jdevries3133/calcount:${data.external.git_describe.result.output}"
  domain    = "beancount.bot"

  extra_env = {
    SESSION_SECRET                = random_password.secret_key.result
    OPENAI_API_KEY                = var.openai_api_key
    STRIPE_API_KEY                = var.stripe_api_key
    STRIPE_WEBHOOK_SIGNING_SECRET = var.stripe_webhook_signing_secret
    SMTP_EMAIL_USERNAME           = "jdevries3133@gmail.com"
    SMTP_EMAIL_PASSWORD           = var.smtp_email_password
  }
}
