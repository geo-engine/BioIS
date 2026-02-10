# BioIS - Biodiversity Indicator Service

[![Build Status](https://github.com/geo-engine/BioIS/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/geo-engine/BioIS/actions)
[![Coverage Status](https://coveralls.io/repos/github/geo-engine/BioIS/badge.svg?branch=main)](https://coveralls.io/github/geo-engine/BioIS?branch=main)

BioIS provides services and tooling to compute, aggregate and serve biodiversity indicators derived from geospatial datasets.
It is designed to power analyses, visualisations and downstream applications by exposing a stable backend API and data-processing components.

## Service

_TODO: Add link to hosted service when available._

## Components

BioIS consists of several key components.

🏗️ Architecture

```mermaid
flowchart TB
  %% Definitions
  ui["Angular UI<br/>(TypeScript)"]
  bindings["TS Bindings<br/>(Generated)"]
  service["Rust Service<br/>(Axum/utoipa)"]

  %% Relations
  ui -->|HTTP/REST| bindings
  bindings -->|Type-safe calls| service
```

### [Backend](backend/README.md)

The core service is implemented in Rust, providing APIs and processing capabilities for biodiversity indicators.

### [API Client](api-client/README.md)

The API client is a TypeScript library generated from the OpenAPI specification of the BioIS backend service.

### [Frontend](frontend/README.md)

_TODO: Add description of frontend component._
