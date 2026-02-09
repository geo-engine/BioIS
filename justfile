_default:
    @just --list

OPENAPI_GENERATOR_CONTAINER := "docker.io/openapitools/openapi-generator-cli:v7.18.0"

[group('api-client')]
[working-directory('api-client')]
validate-api-client:
    clear
    podman run --rm \
        -v ${PWD}/../openapi.json:/local/openapi.json \
        {{ OPENAPI_GENERATOR_CONTAINER }} validate -i /local/openapi.json --recommend

[group('api-client')]
[working-directory('api-client')]
generate-api-client:
    clear
    podman run --rm \
        -v ${PWD}:/local \
        -v ${PWD}/../openapi.json:/local/openapi.json \
        {{ OPENAPI_GENERATOR_CONTAINER }} batch /local/config.yaml

[group('api-client')]
[working-directory('api-client')]
test-generate-api-client:
    podman run --rm -i \
        -v ${PWD}:/local \
        -v ${PWD}/../openapi.json:/local/openapi.json \
        {{ OPENAPI_GENERATOR_CONTAINER }} bash

# Generate the OpenAPI spec and write it to `openapi.json`.
[group('backend')]
[working-directory('backend')]
generate-openapi-spec:
    clear && cargo run --bin openapi > ../openapi.json

# Build the backend. Usage: `just build-backend --release`.
[group('backend')]
[working-directory('backend')]
build-backend mode="":
    clear && cargo build {{ mode }}

# Run the backend. Usage: `just run-backend --release`.
[group('backend')]
[working-directory('backend')]
run-backend mode="":
    clear && cargo run {{ mode }}
