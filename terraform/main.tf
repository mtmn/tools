module "github" {
  source = "./github"

  github_owner = var.github_owner
  github_token = var.github_token

  github_repositories = local.github_repositories
}
