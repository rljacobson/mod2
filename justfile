# List all available just commands
default:
    just --list


# Run all unit and integration tests for all packages
[group("Test")]
test:
    cargo test --workspace --verbose

########################################
# Build tasks
########################################

# Build the default Rust targets (bin and lib) for all workspace members
[group("Build")]
build-workspace:
    cargo build --workspace --verbose

# Build all targets: lib, bin, test, example, bench
[group("Build")]
build-workspace-all:
    cargo build --all-targets --workspace --verbose

# Build the Rust API docs into the default directory `target/docs`, pass `--open`
[group("Build")]
build-docs *rest:
    cargo doc --workspace --no-deps --document-private-items {{ rest }}

########################################
# Docs & Markdown Lints
########################################

# Lint all Markdown files (no fixes)
[group("Lint")]
lint-md: install-markdownlint (lint-md-file '"**/*.md"')

# Auto-fix and format all Markdown files
[group("Lint")]
fix-md:
    @echo "Reformat all markdown files in the project? [y/N]"; \
        read -r confirm && [ "$confirm" = "y" ] || exit 1
    just fix-md-file "**/*.md"

# Lint a specific Markdown `filename`
[group('Lint')]
[no-cd]
lint-md-file filename: install-markdownlint
    markdownlint {{ filename }}

# Fix and format a specific Markdown `filename`
[group('Lint')]
[no-cd]
fix-md-file filename: install-markdownlint install-prettier
    markdownlint --fix {{ filename }} || true
    prettier --write {{ filename }}

# Install markdownlint
[group('Install')]
[group("Lint")]
install-markdownlint:
    npm install -g markdownlint-cli

# Install or upgrade Prettier for formatting Markdown
[group('Install')]
[group("Lint")]
install-prettier:
    npm install -g prettier

# Cargo clean, removes `target` directory
[group("Clean")]
clean-target:
    cargo clean
