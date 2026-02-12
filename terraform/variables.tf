variable "github_repositories" {
  type = map(object({
    description            = string
    visibility             = string
    has_issues             = bool
    has_projects           = bool
    has_wiki               = bool
    has_discussions        = bool
    allow_merge_commit     = bool
    allow_squash_merge     = bool
    allow_rebase_merge     = bool
    allow_auto_merge       = bool
    delete_branch_on_merge = bool
    vulnerability_alerts   = bool
    archive_on_destroy     = bool
    auto_init              = bool
    gitignore_template     = string
    license_template       = string
    homepage_url           = string
    topics                 = list(string)
  }))
  default = {}
}

variable "github_owner" {
  type      = string
  sensitive = true
}

variable "github_token" {
  type      = string
  sensitive = true
}
