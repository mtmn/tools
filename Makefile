.PHONY: update-python-deps update-rust-deps build deploy fetch tidy clean help noop

.DEFAULT_GOAL := help

CARGO ?= cargo
BAZEL ?= bazel

TARGETS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))

update-python-deps: ## Update `requirements.out` lockfile
	$(BAZEL) run //:requirements.update

update-rust-deps: ## Update `Cargo.lock` lockfile
	$(CARGO) generate-lockfile

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

clean: ## Run `bazel clean`
	$(BAZEL) clean

tidy: ## Run `bazel mod tidy`
	$(BAZEL) mod tidy

help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

%: noop ## Avoid "nothing to be done" for any target that doesn't have a rule
	@:
