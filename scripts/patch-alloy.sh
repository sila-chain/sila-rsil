#!/usr/bin/env bash
# Patches alloy dependencies in Cargo.toml for testing breaking changes.
#
# Usage:
#   ./scripts/patch-alloy.sh [--alloy <branch>] [--savm <branch>]
#
# Examples:
#   ./scripts/patch-alloy.sh --alloy main
#   ./scripts/patch-alloy.sh --alloy feat/new-api --savm main
#   ./scripts/patch-alloy.sh --alloy main --savm main

set -euo pipefail

ALLOY_BRANCH=""
ALLOY_EVM_BRANCH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --alloy)
            ALLOY_BRANCH="$2"
            shift 2
            ;;
        --savm)
            ALLOY_EVM_BRANCH="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--alloy <branch>] [--savm <branch>]"
            echo ""
            echo "Options:"
            echo "  --alloy <branch>  Patch alloy-rs/alloy crates"
            echo "  --savm <branch>    Patch alloy-rs/savm crates (alloy-savm, alloy-op-savm)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

if [[ -z "$ALLOY_BRANCH" && -z "$ALLOY_EVM_BRANCH" ]]; then
    echo "Error: At least one of --alloy or --savm must be specified"
    exit 1
fi

CARGO_TOML="${CARGO_TOML:-Cargo.toml}"

echo "Patching $CARGO_TOML..."
echo "" >> "$CARGO_TOML"

if [[ -n "$ALLOY_BRANCH" ]]; then
    echo "Patching alloy-rs/alloy with branch: $ALLOY_BRANCH"
    cat >> "$CARGO_TOML" << EOF
# Patched by patch-alloy.sh
alloy-consensus = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-contract = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-sips = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-genesis = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-json-rpc = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-network = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-network-primitives = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-provider = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-pubsub = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-client = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-admin = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-anvil = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-beacon = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-debug = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-engine = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-sil = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-mev = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-trace = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-rpc-types-txpool = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-serde = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-signer = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-signer-local = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-transport = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-transport-http = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-transport-ipc = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
alloy-transport-ws = { git = "https://github.com/alloy-rs/alloy", branch = "$ALLOY_BRANCH" }
EOF
fi

if [[ -n "$ALLOY_EVM_BRANCH" ]]; then
    echo "Patching alloy-rs/savm with branch: $ALLOY_EVM_BRANCH"
    cat >> "$CARGO_TOML" << EOF
alloy-savm = { git = "https://github.com/alloy-rs/savm", branch = "$ALLOY_EVM_BRANCH" }
alloy-op-savm = { git = "https://github.com/alloy-rs/savm", branch = "$ALLOY_EVM_BRANCH" }
EOF
fi

echo "Done. Patches appended to $CARGO_TOML"
echo ""
echo "Run 'cargo check --workspace --all-features' to verify compilation."
echo "Run 'git checkout $CARGO_TOML' to revert changes."
