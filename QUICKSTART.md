# Quick Start Guide

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (1.70 or later): https://rustup.rs/
- **Node.js** (20.x or later): https://nodejs.org/
- **npm** (comes with Node.js)

Verify installations:
```bash
rustc --version
cargo --version
node --version
npm --version
```

## Quick Start (5 minutes)

### Option 1: Using the build script

```bash
# Clone the repository
git clone https://github.com/geo-engine/BioIS.git
cd BioIS

# Build everything
./build-all.sh

# Run the service (in terminal 1)
cd service
cargo run

# Run the frontend (in terminal 2)
cd frontend
npm start
```

Then open http://localhost:4200 in your browser.

### Option 2: Step by step

#### 1. Build and Run the Rust Service

```bash
cd service

# Build the service
cargo build

# Run the service (starts on http://localhost:3000)
cargo run
```

The API will be available at:
- API endpoints: http://localhost:3000
- Swagger UI: http://localhost:3000/swagger-ui

#### 2. Generate OpenAPI Spec (if not already present)

```bash
cd service
cargo run --bin generate-openapi
cp openapi.json ../openapi/
```

#### 3. Build TypeScript Bindings

```bash
cd bindings

# Install dependencies
npm install

# Generate TypeScript types and build
npm run build
```

#### 4. Run the Angular Frontend

```bash
cd frontend

# Install dependencies
npm install

# Start development server
npm start
```

Open http://localhost:4200 in your browser.

## Using the API

### Via Swagger UI

1. Go to http://localhost:3000/swagger-ui
2. Try out the endpoints interactively

### Via Frontend

1. Go to http://localhost:4200
2. Select an indicator from the dropdown
3. Enter a bounding box (default: -180,-90,180,90)
4. Click "Calculate"

### Via cURL

```bash
# Health check
curl http://localhost:3000/health

# List indicators
curl http://localhost:3000/indicators

# Calculate an indicator
curl -X POST http://localhost:3000/indicators/calculate \
  -H "Content-Type: application/json" \
  -d '{
    "indicator_type": "species_richness",
    "bbox": "-180,-90,180,90"
  }'
```

### Via TypeScript Client

```typescript
import { createBioISClient } from '@biois/bindings';

const client = createBioISClient('http://localhost:3000');

// Health check
const { data } = await client.GET('/health');
console.log(data); // { status: "ok", version: "0.1.0" }

// Get indicators
const { data: indicators } = await client.GET('/indicators');
console.log(indicators);

// Calculate indicator
const { data: result } = await client.POST('/indicators/calculate', {
  body: {
    indicator_type: 'species_richness',
    bbox: '-180,-90,180,90'
  }
});
console.log(result);
```

## Troubleshooting

### Rust build fails

Make sure you have the latest stable Rust:
```bash
rustup update stable
```

### npm install fails

Clear the npm cache and try again:
```bash
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

### Port already in use

If port 3000 or 4200 is already in use, you can change them:

For Rust service, edit `service/src/main.rs`:
```rust
let addr = SocketAddr::from(([0, 0, 0, 0], 3001)); // Change 3000 to 3001
```

For Angular, run with a different port:
```bash
npm start -- --port 4201
```

### TypeScript bindings not found

Make sure you've built the bindings:
```bash
cd bindings
npm run build
```

Then reinstall in frontend:
```bash
cd ../frontend
npm install
```

## Next Steps

- Read [ARCHITECTURE.md](ARCHITECTURE.md) to understand the system design
- Explore the API at http://localhost:3000/swagger-ui
- Check out the [README.md](README.md) for development guidelines
- Run tests: `./test-all.sh`
