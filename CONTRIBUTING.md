# Contributing to BioIS

Thank you for your interest in contributing to BioIS! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/BioIS.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test your changes
6. Commit and push
7. Open a Pull Request

## Development Setup

See [QUICKSTART.md](QUICKSTART.md) for detailed setup instructions.

## Project Structure

```
BioIS/
├── service/              # Rust backend service
│   ├── src/
│   │   ├── lib.rs       # Library with API logic
│   │   ├── main.rs      # Binary entry point
│   │   └── bin/
│   │       └── generate_openapi.rs  # OpenAPI generator
│   └── Cargo.toml
├── bindings/            # TypeScript OpenAPI bindings
│   ├── src/
│   │   └── index.ts     # Client wrapper
│   ├── package.json
│   └── tsconfig.json
├── frontend/            # Angular web application
│   ├── src/
│   │   └── app/
│   │       ├── app.ts           # Main component
│   │       └── biois.service.ts # API service
│   ├── package.json
│   └── angular.json
└── openapi/            # Shared OpenAPI specification
    └── openapi.json
```

## Making Changes

### Rust Service

1. Make your changes in `service/src/`
2. Format: `cargo fmt`
3. Lint: `cargo clippy`
4. Test: `cargo test`
5. Update OpenAPI spec: `cargo run --bin generate-openapi`

When adding new endpoints:
- Add utoipa annotations (`#[utoipa::path(...)]`)
- Update the `ApiDoc` struct in `lib.rs`
- Document request/response types with `ToSchema`

Example:
```rust
#[derive(Serialize, Deserialize, ToSchema)]
struct MyRequest {
    /// Field description
    #[schema(example = "example value")]
    field: String,
}

#[utoipa::path(
    post,
    path = "/my-endpoint",
    request_body = MyRequest,
    responses(
        (status = 200, description = "Success", body = MyResponse)
    )
)]
async fn my_endpoint(Json(req): Json<MyRequest>) -> Json<MyResponse> {
    // Implementation
}
```

### TypeScript Bindings

The bindings are auto-generated. If you need to modify the client:

1. Update `bindings/src/index.ts` for wrapper functions
2. Rebuild: `npm run build`

### Angular Frontend

1. Make your changes in `frontend/src/`
2. Format/lint: `npm run lint` (if configured)
3. Test: `npm test`
4. Build: `npm run build`

When using new API endpoints:
1. The types are automatically available from `@biois/bindings`
2. Add methods to `biois.service.ts` if needed
3. Use the typed client in components

Example:
```typescript
import { createBioISClient } from '@biois/bindings';
import type { components } from '@biois/bindings';

type MyRequest = components['schemas']['MyRequest'];
type MyResponse = components['schemas']['MyResponse'];

async myMethod(req: MyRequest): Promise<MyResponse> {
  const { data, error } = await this.client.POST('/my-endpoint', {
    body: req
  });
  if (error) throw error;
  return data!;
}
```

## Testing

### Unit Tests

```bash
# Rust
cd service && cargo test

# Frontend
cd frontend && npm test
```

### Integration Tests

```bash
./test-all.sh
```

### Manual Testing

1. Start the service: `cd service && cargo run`
2. Start the frontend: `cd frontend && npm start`
3. Test in browser at http://localhost:4200

## Code Style

### Rust
- Use `cargo fmt` for formatting
- Follow Rust standard naming conventions
- Add documentation comments for public APIs
- Run `cargo clippy` and fix warnings

### TypeScript/Angular
- Use consistent indentation (2 spaces)
- Follow Angular style guide
- Use meaningful variable names
- Add JSDoc comments for complex functions

## Commit Messages

Use clear, descriptive commit messages:

```
feat: Add new biodiversity indicator endpoint
fix: Correct bounding box validation
docs: Update API documentation
test: Add tests for indicator calculation
refactor: Simplify service initialization
```

Prefixes:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test changes
- `refactor`: Code refactoring
- `chore`: Build/tooling changes

## Pull Request Process

1. Update documentation if needed
2. Ensure all tests pass
3. Update CHANGELOG.md if applicable
4. Request review from maintainers
5. Address review comments
6. Squash commits if requested

## Questions?

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Tag issues appropriately

Thank you for contributing!
