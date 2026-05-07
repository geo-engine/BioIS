_default:
    @just --list

OPENAPI_GENERATOR_PACKAGE := "@openapitools/openapi-generator-cli@2.28.3"

# Clear the terminal before executing a command. Does not fail in a CI.
_clear:
    @-clear

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
install-frontend-deps: _clear
    rm -rf node_modules/@geoengine/biois \
           .angular/cache
    npm link ../api-client/typescript
    npm ci

[group('frontend')]
[group('install')]
[working-directory('api-client/typescript')]
install-api-client-deps: _clear
    npm install

### BUILD ###

# Build the backend. Usage: `just build-backend --release`.
[group('backend')]
[group('build')]
[working-directory('backend')]
build-backend mode="": _clear
    cargo build {{ mode }}

[group('api-client')]
[group('build')]
[working-directory('api-client')]
build-api-client: _clear
    npx {{ OPENAPI_GENERATOR_PACKAGE }} batch config.yaml
    ./post-process.py

# Build the backend. Usage: `just build-backend --release`.
[group('build')]
[group('frontend')]
[working-directory('frontend')]
build-frontend: _clear
    npm ci
    npm run build

# Build the frontend container. Usage: `just build-frontend-container`.
[group('build')]
[group('container')]
build-frontend-container: _clear
    podman build \
        -f frontend/Dockerfile \
        -t biois-frontend:latest \
        .

# Build the frontend container. Usage: `just build-frontend-container`.
[group('build')]
[group('container')]
build-backend-container: _clear
    podman build \
        -f backend/Dockerfile \
        -t biois-backend:latest \
        backend

### LINT ###

[group('api-client')]
[group('lint')]
[working-directory('api-client')]
lint-openapi-spec: _clear
    npx {{ OPENAPI_GENERATOR_PACKAGE }} validate -i ../openapi.json --recommend

[group('api-client')]
[group('lint')]
[working-directory('api-client/typescript')]
lint-api-client: _clear
    npm install
    npm run build
    rm -r node_modules
    rm package-lock.json

[group('backend')]
[group('lint')]
[working-directory('backend')]
lint-backend-rustfmt: _clear
    @echo "Running rustfmt…"
    cargo fmt --all -- --check

[arg("relaxed", long="relaxed", value="true", help="Don't fail on warnings, only print them")]
[group('backend')]
[group('lint')]
[working-directory('backend')]
lint-backend-clippy relaxed="false": _clear
    @echo "Running clippy…"
    cargo clippy --all-targets --locked {{ if relaxed == "true" { "" } else { "-- -D warnings" } }}

[group('backend')]
[group('lint')]
[working-directory('backend')]
lint-backend-sqlfluff: _clear
    echo "Running sqlfluff…"
    pipx run sqlfluff==3.5.0 lint

[group('backend')]
[group('lint')]
[working-directory('backend')]
lint-backend-diesel-cli: _clear
    @echo "Running Diesel CLI…"
    diesel migration run --locked-schema
    diesel-guard check migrations

[group('backend')]
[group('lint')]
lint-backend: lint-backend-rustfmt lint-backend-clippy lint-backend-sqlfluff lint-backend-diesel-cli

[group('frontend')]
[group('lint')]
[working-directory('frontend')]
lint-frontend-fmt: _clear
    npm run prettier -- --check .

[group('frontend')]
[group('lint')]
[working-directory('frontend')]
lint-frontend-code: _clear
    npm run lint

[group('frontend')]
[group('lint')]
lint-frontend: lint-frontend-fmt lint-frontend-code

### TEST ###

# Run the backend tests. Usage: `just test-backend`.
[group('backend')]
[group('test')]
[working-directory('backend')]
test-backend: _clear
    cargo test --locked --all-features

# Run the backend tests and generate coverage. Usage: `just test-backend-with-coverage`.
[group('backend')]
[group('test')]
[working-directory('backend')]
test-backend-with-coverage outputPath="lcov.info": _clear
    cargo llvm-cov \
        --locked \
        --all-features \
        --lcov \
        --output-path {{ outputPath }}

# Run the backend tests and generate coverage. Usage: `just test-backend-with-coverage`.
[group('backend')]
[group('test')]
[working-directory('backend')]
test-backend-with-coverage-report: _clear
    cargo llvm-cov \
        --locked \
        --all-features \
        --html

# Run the frontend tests. Usage: `just test-frontend`.
[group('frontend')]
[group('test')]
[working-directory('frontend')]
test-frontend: _clear
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
run-backend mode="": _clear
    cargo run {{ mode }}

# Run the backend. Usage: `just run-frontend`.
[group('frontend')]
[group('run')]
[working-directory('frontend')]
run-frontend: _clear
    npm run ng serve

# Run the backend container in dev mode. Usage: `just run-backend-container`.
[group('container')]
[group('run')]
run-backend-container: build-backend-container
    podman run --rm --replace \
        --name biois-backend-dev \
        --network host \
        -p 4040:4040 \
        -v $(pwd)/backend/Settings.toml:/usr/local/bin/Settings.toml \
        biois-backend:latest

# Run the frontend container in dev mode. Usage: `just run-frontend-container`.
[group('container')]
[group('run')]
run-frontend-container: build-frontend-container
    podman run --rm --replace \
        --name biois-frontend-dev \
        -p 4200:80 \
        biois-frontend:latest

# Run the container as a pod in dev mode. Usage: `just run-pod`.
[group('container')]
[group('run')]
run-pod: build-backend-container build-frontend-container
    cat k8s/dev-config.yaml k8s/pod.yaml | \
    podman play kube \
        --network=pasta:-T,3030:3030 `# Map local Geo Engine at port 3030 into pod` \
        --replace -

# Stop the pod in dev mode. Usage: `just down-pod`.
[group('container')]
[group('run')]
down-pod:
    cat k8s/dev-config.yaml k8s/pod.yaml | \
    podman play kube \
        --down -

### MISC ###

# Generate the OpenAPI spec and write it to `openapi.json`.
[group('backend')]
[group('misc')]
[working-directory('backend')]
generate-openapi-spec: _clear
    cargo run --bin openapi > ../openapi.json

# Run an arbitrary Angular CLI command in the frontend. Usage: `just ng -- build`.
[group('frontend')]
[group('misc')]
[working-directory('frontend')]
ng +ARGS="": _clear
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
