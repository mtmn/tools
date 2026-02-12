output "github_repo_urls" {
  value = {
    for name, repo in github_repository.repo : name => repo.html_url
  }
}

output "github_repo_ssh_urls" {
  value = {
    for name, repo in github_repository.repo : name => repo.ssh_clone_url
  }
}
