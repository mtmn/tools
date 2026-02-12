provider "github" {
  token = var.github_token
  owner = var.github_owner
}

resource "github_repository" "repo" {
  for_each = var.github_repositories

  name        = each.key
  description = each.value.description

  visibility = each.value.visibility

  has_issues      = each.value.has_issues
  has_projects    = each.value.has_projects
  has_wiki        = each.value.has_wiki
  has_discussions = each.value.has_discussions

  allow_merge_commit     = each.value.allow_merge_commit
  allow_squash_merge     = each.value.allow_squash_merge
  allow_rebase_merge     = each.value.allow_rebase_merge
  allow_auto_merge       = each.value.allow_auto_merge
  delete_branch_on_merge = each.value.delete_branch_on_merge

  vulnerability_alerts = each.value.vulnerability_alerts
  archive_on_destroy   = each.value.archive_on_destroy

  auto_init          = each.value.auto_init
  gitignore_template = each.value.gitignore_template
  license_template   = each.value.license_template

  homepage_url = each.value.homepage_url
  topics       = length(each.value.topics) > 0 ? each.value.topics : null
}
