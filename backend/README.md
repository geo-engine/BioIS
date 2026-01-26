# Backend

This directory contains the backend service for BioIS, implemented in Rust.
It provides APIs and processing capabilities for computing biodiversity indicators from geospatial datasets.

## Setup

1. Ensure you have Rust installed.
   You can install it via [rustup](https://rustup.rs/).

2. Ensure PostgreSQL is installed and running.
   Configure the database connection settings in the environment variables or configuration files as needed.

3. Run tests to ensure everything is set up correctly:

   ```bash
    cargo test
   ```

4. Apply linting and formatting checks:

   ```bash
   ./lint.sh
   ```

5. Run the backend service locally:

   ```bash
    cargo run
   ```

## Configuration

The backend service can be configured via environment variables or configuration files.

_TODO: Add detailed configuration instructions here._
