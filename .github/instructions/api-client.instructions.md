---
applyTo: "api-client/**/*"
---

# Copilot Instructions

- Never modify the `typescript` directory manually.
  It is generated from the OpenAPI specification in `openapi.json` using the OpenAPI Generator.
  To regenerate the API client, run `just generate-api-client` from the project root.
- The OpenAPI Generator configuration is defined in `config.yaml`.
  If you need to change the generated code, modify the configuration and regenerate the client.
