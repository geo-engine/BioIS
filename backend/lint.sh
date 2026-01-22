#!/bin/env bash

set -euo pipefail

function rustfmt {
    cargo fmt --all -- --check
}

function clippy {
    cargo clippy --all-targets --locked -- -D warnings
}

function sqlfluff {
    pipx run sqlfluff==3.5.0 lint
}

function diesel_cli {
    local should_install=${1:-}
    
    if [ "$should_install" == "--with-install" ]; then
        echo "Installing Diesel CLI and Diesel Guard…"
        cargo install diesel_cli --no-default-features --features postgres
        cargo install diesel-guard --no-default-features
    fi

    diesel migration run --locked-schema
    diesel-guard check migrations
}

function all {
    echo "Running rustfmt…"
    rustfmt
    echo "Running clippy…"
    clippy
    echo "Running sqlfluff…"
    sqlfluff
    echo "Running Diesel CLI…"
    diesel_cli $@
}

function help {
    echo "$0 <task> <args>"
    echo "Tasks:"
    compgen -A function | grep -e '^_' -v | cat -n
    echo "Args:"
    echo "  --with-install    Install Diesel CLI and Diesel Guard before running migrations"
}

function _default {
    help
}

${@:-_default}
