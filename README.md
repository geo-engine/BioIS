# BioIS
Biodiversity Indicator Service

A monorepo containing:
- **Rust Service**: Backend API for calculating biodiversity indicators using GeoEngine API
- **TypeScript Bindings**: Auto-generated OpenAPI TypeScript client
- **Angular Frontend**: Web interface for interacting with the service

## Project Structure

```
.
├── service/          # Rust biodiversity indicator service
├── openapi/          # OpenAPI specification
├── bindings/         # TypeScript bindings generated from OpenAPI spec
├── frontend/         # Angular web application
└── .github/          # CI/CD workflows
```

## Getting Started

### Prerequisites

- Rust (1.70+)
- Node.js (20+)
- npm (10+)

### Building the Rust Service

```bash
cd service
cargo build
cargo run
```

The service will start on `http://localhost:3000`

Available endpoints:
- `GET /health` - Health check
- `GET /indicators` - List available biodiversity indicators
- `POST /indicators/calculate` - Calculate indicator for a given area
- `GET /swagger-ui` - Interactive API documentation

### Generating OpenAPI Spec

```bash
cd service
cargo run --bin generate-openapi
```

This creates `openapi.json` which is copied to the `openapi/` directory.

### Building TypeScript Bindings

```bash
cd bindings
npm install
npm run build
```

This generates TypeScript types and client from the OpenAPI specification.

### Building the Angular Frontend

```bash
cd frontend
npm install
npm run build
npm start
```

The frontend will be available at `http://localhost:4200`

## Development

### Running Tests

```bash
# Rust tests
cd service && cargo test

# Bindings tests
cd bindings && npm test

# Frontend tests
cd frontend && npm test
```

### Code Quality

```bash
# Rust formatting and linting
cd service
cargo fmt
cargo clippy

# TypeScript/Angular linting
cd frontend
npm run lint
```

## CI/CD

The project uses GitHub Actions for continuous integration:
- Builds and tests the Rust service
- Generates TypeScript bindings
- Builds and tests the Angular frontend

See `.github/workflows/ci.yml` for details.

## License

MIT
