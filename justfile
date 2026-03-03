_default:
    @just --list

OPENAPI_GENERATOR_PACKAGE := "@openapitools/openapi-generator-cli@2.28.3"

### INSTALL ###

[group('backend')]
[group('install')]
[working-directory('backend')]
install-diesel-cli:
    @echo "Installing Diesel CLI and Diesel Guard…"
    cargo install diesel_cli --no-default-features --features postgres
    cargo install diesel-guard --no-default-features

[group('backend')]
[group('install')]
[working-directory('backend')]
install-llvm-cov:
    @echo "Installing llvm-cov…"
    cargo install --locked cargo-llvm-cov

[group('frontend')]
[group('install')]
[working-directory('frontend')]
install-frontend-deps:
    @-clear
    rm -rf node_modules/@geoengine/biois \
           .angular/cache
    npm link ../api-client/typescript
    npm ci

[group('frontend')]
[group('install')]
[working-directory('api-client/typescript')]
install-api-client-deps:
    @-clear
    npm install

### BUILD ###

# Build the backend. Usage: `just build-backend --release`.
[group('backend')]
[group('build')]
[working-directory('backend')]
build-backend mode="":
    @-clear
    cargo build {{ mode }}

[group('api-client')]
[group('build')]
[working-directory('api-client')]
build-api-client:
    @-clear
    npx {{ OPENAPI_GENERATOR_PACKAGE }} batch config.yaml
    ./post-process.py

# Build the backend. Usage: `just build-backend --release`.
[group('build')]
[group('frontend')]
[working-directory('frontend')]
build-frontend:
    @-clear
    npm ci
    npm run build

### LINT ###

[group('api-client')]
[group('lint')]
[working-directory('api-client')]
lint-openapi-spec:
    @-clear
    npx {{ OPENAPI_GENERATOR_PACKAGE }} validate -i ../openapi.json --recommend

[group('api-client')]
[group('lint')]
[working-directory('api-client/typescript')]
lint-api-client:
    @-clear
    npm install
    npm run build
    rm -r node_modules
    rm package-lock.json

[group('backend')]
[group('lint')]
[working-directory('backend')]
lint-backend-rustfmt:
    cargo fmt --all -- --check

[group('backend')]
[group('lint')]
[working-directory('backend')]
lint-backend-clippy:
    cargo clippy --all-targets --locked -- -D warnings

[group('backend')]
[group('lint')]
[working-directory('backend')]
lint-backend-sqlfluff:
    pipx run sqlfluff==3.5.0 lint

[group('backend')]
[group('lint')]
[working-directory('backend')]
lint-backend-diesel-cli:
    diesel migration run --locked-schema
    diesel-guard check migrations

[group('backend')]
[group('lint')]
lint-backend:
    @-clear
    @echo "Running rustfmt…"
    just lint-backend-rustfmt
    @echo "Running clippy…"
    just lint-backend-clippy
    @echo "Running sqlfluff…"
    just lint-backend-sqlfluff
    @echo "Running Diesel CLI…"
    just lint-backend-diesel-cli

[group('frontend')]
[group('lint')]
[working-directory('frontend')]
lint-frontend-fmt:
    npm run prettier -- --check .

[group('frontend')]
[group('lint')]
[working-directory('frontend')]
lint-frontend-code:
    npm run lint

[group('frontend')]
[group('lint')]
lint-frontend:
    @-clear
    just lint-frontend-fmt
    just lint-frontend-code

### TEST ###

# Run the backend tests. Usage: `just test-backend`.
[group('backend')]
[group('test')]
[working-directory('backend')]
test-backend:
    @-clear
    cargo test --locked --all-features

# Run the backend tests and generate coverage. Usage: `just test-backend-with-coverage`.
[group('backend')]
[group('test')]
[working-directory('backend')]
test-backend-with-coverage outputPath="lcov.info":
    @-clear
    cargo llvm-cov \
        --locked \
        --all-features \
        --lcov \
        --output-path {{ outputPath }}

# Run the backend tests and generate coverage. Usage: `just test-backend-with-coverage`.
[group('backend')]
[group('test')]
[working-directory('backend')]
test-backend-with-coverage-report:
    @-clear
    cargo llvm-cov \
        --locked \
        --all-features \
        --html

# Run the frontend tests. Usage: `just test-frontend`.
[group('frontend')]
[group('test')]
[working-directory('frontend')]
test-frontend:
    @-clear
    npm run test

### RUN ###

# Run backend and frontend at the same time. Usage: `just run`.
[group('run')]
[parallel]
run: run-backend run-frontend

# Run the backend. Usage: `just run-backend --release`.
[group('backend')]
[group('run')]
[working-directory('backend')]
run-backend mode="":
    @-clear
    cargo run {{ mode }}

# Run the backend. Usage: `just run-frontend`.
[group('frontend')]
[group('run')]
[working-directory('frontend')]
run-frontend:
    @-clear
    npm run ng serve

### MISC ###

# Generate the OpenAPI spec and write it to `openapi.json`.
[group('backend')]
[group('misc')]
[working-directory('backend')]
generate-openapi-spec:
    @-clear
    cargo run --bin openapi > ../openapi.json

# Run an arbitrary Angular CLI command in the frontend. Usage: `just ng -- build`.
[group('frontend')]
[group('misc')]
[working-directory('frontend')]
ng +ARGS="":
    @-clear
    npm run ng -- {{ ARGS }}

check-no-changes-in-git-repo:
    #!/usr/bin/env bash
    if [ -n "$(git status --porcelain)" ]; then
      echo "Error: Uncommitted changes found in git repository."
      git status --porcelain
      exit 1
    else
      echo "No uncommitted changes found in git repository."
    fi
