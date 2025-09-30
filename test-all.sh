#!/bin/bash
set -e

echo "=== Testing BioIS Monorepo ==="
echo ""

echo "1. Building Rust service..."
cd service
cargo build
cargo test
echo "✓ Rust service built and tested"
echo ""

echo "2. Generating OpenAPI spec..."
cargo run --bin generate-openapi
cp openapi.json ../openapi/
echo "✓ OpenAPI spec generated"
echo ""

echo "3. Building TypeScript bindings..."
cd ../bindings
npm run build
echo "✓ TypeScript bindings built"
echo ""

echo "4. Building Angular frontend..."
cd ../frontend
npm run build
echo "✓ Angular frontend built"
echo ""

echo "=== All tests passed! ==="
