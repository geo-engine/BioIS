_default:
    @just --list

OPENAPI_GENERATOR_CONTAINER := "docker.io/openapitools/openapi-generator-cli:v7.18.0"

### BUILD ###

# Build the backend. Usage: `just build-backend --release`.
[group('backend')]
[group('build')]
[working-directory('backend')]
build-backend mode="":
    clear && cargo build {{ mode }}

[group('api-client')]
[group('build')]
[working-directory('api-client')]
build-api-client:
    clear
    podman run --rm \
        -v ${PWD}:/local \
        -v ${PWD}/../openapi.json:/local/openapi.json \
        {{ OPENAPI_GENERATOR_CONTAINER }} batch /local/config.yaml

### LINT ###

[group('api-client')]
[group('lint')]
[working-directory('api-client')]
lint-openapi-spec:
    clear
    podman run --rm \
        -v ${PWD}/../openapi.json:/local/openapi.json \
        {{ OPENAPI_GENERATOR_CONTAINER }} validate -i /local/openapi.json --recommend

[group('api-client')]
[group('lint')]
[working-directory('api-client/typescript')]
lint-api-client:
    clear
    npm install
    npm run build
    # rm -r node_modules
    # rm package-lock.json

### TEST ###

# Run the backend tests. Usage: `just test-backend`.
[group('backend')]
[group('test')]
[working-directory('backend')]
test-backend:
    clear && cargo test

# Run the frontend tests. Usage: `just test-frontend`.
[group('frontend')]
[group('test')]
[working-directory('frontend')]
test-frontend:
    clear && npm run ng test

### RUN ###

# Run the backend. Usage: `just run-backend --release`.
[group('backend')]
[group('run')]
[working-directory('backend')]
run-backend mode="":
    clear && cargo run {{ mode }}

# Run the backend. Usage: `just run-frontend`.
[group('frontend')]
[group('run')]
[working-directory('frontend')]
run-frontend:
    clear && npm run ng serve

### MISC ###

# Generate the OpenAPI spec and write it to `openapi.json`.
[group('backend')]
[group('misc')]
[working-directory('backend')]
generate-openapi-spec:
    clear && cargo run --bin openapi > ../openapi.json

# Run an arbitrary Angular CLI command in the frontend. Usage: `just ng -- build`.
[group('frontend')]
[group('misc')]
[working-directory('frontend')]
ng +ARGS="":
    clear && npm run ng -- {{ ARGS }}
