.PHONY: all build format lint check clean

# Default target
all: build

# Build the escript binary
build:
	mix deps.get
	mix escript.build

# Format the code
format:
	mix format

# Run the linter
lint:
	mix credo --strict

# Run static type analysis
check:
	mix dialyzer

# Clean build artifacts
clean:
	mix clean
	rm -f khal_notifier
