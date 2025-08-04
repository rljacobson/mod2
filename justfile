# Run all unit and integration tests for all packages (excludes examples and wasm)
test:
    cargo test --workspace --verbose

# Lint all Markdown files (no fixes)
lint-md:
    markdownlint "**/*.md"

# Auto-fix and format all Markdown files
fix-md:
    markdownlint --fix "**/*.md"
    prettier --write "**/*.md"

# Lint a specific Markdown `filename`
lint-md-file filename:
    markdownlint {{filename}}

# Fix and format a specific Markdown `filename`
fix-md-file filename:
    markdownlint --fix {{filename}}
    prettier --write {{filename}}
