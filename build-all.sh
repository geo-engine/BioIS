#!/bin/bash
set -e

echo "=== Building BioIS Monorepo ==="
echo ""

echo "1. Building Rust service..."
cd service
cargo build --release
echo "✓ Rust service built"

echo "2. Generating OpenAPI spec..."
cargo run --release --bin generate-openapi
cp openapi.json ../openapi/
echo "✓ OpenAPI spec generated"

echo "3. Building TypeScript bindings..."
cd ../bindings
npm install
npm run build
echo "✓ TypeScript bindings built"

echo "4. Building Angular frontend..."
cd ../frontend
npm install
npm run build
echo "✓ Angular frontend built"

echo ""
echo "=== Build complete! ==="
echo ""
echo "To run the service:"
echo "  cd service && cargo run"
echo ""
echo "To run the frontend (in development mode):"
echo "  cd frontend && npm start"
