# Copilot Instructions

This BioIS monorepo is written in Rust, SQL, TypeScript and Python.

## Repository Structure

| Path           | Description                                     |
| -------------- | ----------------------------------------------- |
| `README.md`    | Project Documentation                           |
| `LICENSE`      | License File                                    |
| `.github/`     | GitHub Configuration                            |
| `justfile`     | Justfile for task automation                    |
| `backend/`     | Rust API Code                                   |
| `frontend/`    | TypeScript Angular Frontend Code                |
| `api-client/`  | OpenAPI client code generated from the API spec |
| `test-client/` | Snippet-based test client for testing the API   |

- Commit messages and pull request titles should be in the conventional commit format of `<type>(<scope>): <description>`, where:
  - `<type>` is one of `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`.
  - `<scope>` is a noun describing the section of the codebase affected (e.g., `backend`, `frontend`, `api`).
  - `<description>` is a short summary of the change.
- Diagrams should be created using Mermaid syntax and included in the relevant Markdown files for documentation purposes.
