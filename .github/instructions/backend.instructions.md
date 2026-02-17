---
applyTo: "backend/**/*.rs, backend/**/*.sql"
---

# Copilot Instructions

## Coding Style

### Rust

- Follow the Rust API guidelines for naming conventions, error handling, and documentation.
- Use `snake_case` for variable and function names, and `PascalCase` for struct and enum names.
- Ensure that all public functions and types have documentation comments using `///`.
- Use `Result<T>` (from anyhow) for error handling and define custom error types where appropriate.
- Use `camelCase` for JSON field names when serializing/deserializing with serde, and use `#[serde(rename_all = "camelCase")]` on structs to enforce this convention.
- Never use `unwrap()` or `expect()` in production code. Instead, propagate errors using the `?` operator or handle them gracefully.
- Start test function names with `it_` and use descriptive names that indicate what the test is verifying.

### SQL

- The SQL dialect used in this project is PostgreSQL. Follow the PostgreSQL SQL style guide for formatting and conventions.
- For SQL queries, use uppercase for SQL keywords (e.g., SELECT, FROM, WHERE) and lowercase for table and column names.
- SQLFluff is used for linting SQL files. Ensure that your SQL code adheres to the configured SQLFluff rules.
