# Generic Rust workspace tasks.
# Dioxus-specific commands are included because this repo is likely to use Dioxus.

default: check

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

tokei-check:
    ./tokei_check.sh

tokei-check-editor:
    ./tokei_check.sh 800 500 apps/wishing-editor/src

check:
    cargo check --workspace --all-targets --all-features

test:
    cargo test --workspace --all-features

nextest:
    cargo nextest run --workspace

fix:
    cargo clippy --workspace --fix --allow-dirty --allow-staged --all-features

deny:
    cargo deny check

audit:
    cargo audit

clean:
    cargo clean

run:
    cargo run

dioxus-serve:
    dx serve

ssh-preview:
    ./scripts/ssh-preview.sh

dioxus-build:
    dx build

dioxus-bundle:
    dx bundle

ui-compare:
    ./scripts/ui_compare.sh
