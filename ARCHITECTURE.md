# Architecture

## Overview

BioIS is structured as a monorepo with three main components:

```
┌─────────────────────────────────────────┐
│                                         │
│        Angular Frontend (TypeScript)    │
│                                         │
└──────────────┬──────────────────────────┘
               │
               │ HTTP/REST
               │
┌──────────────▼──────────────────────────┐
│                                         │
│     TypeScript OpenAPI Client           │
│     (Auto-generated from spec)          │
│                                         │
└──────────────┬──────────────────────────┘
               │
               │ Type-safe API calls
               │
┌──────────────▼──────────────────────────┐
│                                         │
│      Rust Service (Axum + utoipa)       │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  OpenAPI/Swagger UI             │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  Biodiversity Indicators API    │   │
│  │  - Health check                 │   │
│  │  - List indicators              │   │
│  │  - Calculate indicators         │   │
│  └─────────────────────────────────┘   │
│                                         │
└─────────────────────────────────────────┘
```

## Components

### 1. Rust Service (`service/`)

The backend service built with:
- **Axum**: Modern, ergonomic web framework
- **utoipa**: OpenAPI spec generation from Rust code
- **utoipa-swagger-ui**: Embedded Swagger UI for API documentation
- **tokio**: Async runtime

Key features:
- Type-safe API definitions
- Automatic OpenAPI spec generation
- Built-in Swagger UI at `/swagger-ui`
- CORS enabled for development

### 2. TypeScript Bindings (`bindings/`)

Auto-generated TypeScript client using:
- **openapi-typescript**: Generates TypeScript types from OpenAPI spec
- **openapi-fetch**: Type-safe fetch client

Benefits:
- Full type safety from backend to frontend
- Compile-time error detection
- Auto-completion in IDEs
- Single source of truth (OpenAPI spec)

### 3. Angular Frontend (`frontend/`)

Modern Angular application using:
- Angular 19+ with standalone components
- Signal-based state management
- TypeScript
- Responsive CSS

Features:
- Interactive UI for biodiversity calculations
- Service layer using generated TypeScript client
- Form validation
- Error handling

## Data Flow

1. **Development Time:**
   - Rust code defines API with utoipa annotations
   - OpenAPI spec is generated from Rust code
   - TypeScript types are generated from OpenAPI spec
   - Angular code uses typed client for API calls

2. **Runtime:**
   - User interacts with Angular UI
   - Angular calls TypeScript client methods
   - TypeScript client makes HTTP requests to Rust service
   - Rust service processes requests and returns JSON
   - TypeScript client parses response with type safety
   - Angular UI displays results

## Build Process

```bash
# 1. Build Rust service (generates binary)
cd service && cargo build

# 2. Generate OpenAPI spec (from Rust code)
cargo run --bin generate-openapi

# 3. Generate TypeScript client (from OpenAPI spec)
cd ../bindings && npm run generate

# 4. Build Angular app (using TypeScript client)
cd ../frontend && npm run build
```

## Testing Strategy

- **Rust**: Unit tests with cargo test
- **TypeScript Bindings**: Basic smoke tests
- **Angular**: Component and service tests with Jasmine/Karma
- **Integration**: CI pipeline tests entire build chain

## Future Enhancements

- [ ] Add geoengine API integration
- [ ] Add georust/ogcapi for OGC API Features support
- [ ] Add database for storing calculation results
- [ ] Add authentication/authorization
- [ ] Add real biodiversity calculation algorithms
- [ ] Add Docker support for deployment
- [ ] Add more comprehensive tests
