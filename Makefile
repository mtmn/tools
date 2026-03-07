.PHONY: rust-gen-project pip-deps-update cargo-deps-update clippy rustfmt run build deploy fetch tidy clean help noop

.DEFAULT_GOAL := help

CARGO ?= cargo
BAZEL ?= bazel

TARGETS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))

rust-gen-project: ## Generate schema for `rust-analyzer`
	$(BAZEL) run @rules_rust//tools/rust_analyzer:gen_rust_project

pip-deps-update: ## Update `requirements.out` lockfile
	$(BAZEL) run //:requirements.update

cargo-deps-update: ## Update `Cargo.lock` lockfile
	$(CARGO) generate-lockfile

rust-clippy: ## Run `cargo check`
	@if [ -z "$(TARGETS)" ]; then \
		$(MAKE) help; \
		exit 1; \
	else \
		$(BAZEL) build --aspects=@rules_rust//rust:defs.bzl%rust_clippy_aspect $(addprefix //,$(TARGETS)); \
	fi

rust-rustfmt: ## Format with `rustfmt`
	$(BAZEL) build --aspects=@rules_rust//rust:defs.bzl%rust_clippy_aspect $(addprefix //,$(TARGETS));

run: ## Run `bazel run //<target>`
	@if [ -z "$(TARGETS)" ]; then \
		$(MAKE) help; \
		exit 1; \
	else \
		$(BAZEL) run $(addprefix //,$(TARGETS)); \
	fi

build: ## Run `bazel build //<target>`
	@if [ -z "$(TARGETS)" ]; then \
		$(MAKE) help; \
		exit 1; \
	else \
		$(BAZEL) build $(addprefix //,$(TARGETS)); \
	fi

deploy: ## Run `bazel build //<target>:deploy`
	@if [ -z "$(TARGETS)" ]; then \
		$(MAKE) help; \
		exit 1; \
	else \
		$(BAZEL) run $(addsuffix :deploy,$(addprefix //,$(TARGETS))); \
	fi

fetch: ## Run `bazel fetch //...`
	$(BAZEL) fetch //...

clean: ## Run `bazel clean --expunge`
	$(BAZEL) clean --expunge

tidy: ## Run `bazel mod tidy`
	$(BAZEL) mod tidy

help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

%: noop ## Avoid "nothing to be done" for any target that doesn't have a rule
	@:
