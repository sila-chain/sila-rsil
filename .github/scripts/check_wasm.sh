#!/usr/bin/env bash
set -uxo pipefail

readarray -t crates < <(
  cargo metadata --format-version=1 --no-deps | jq -r '.packages[].name' | grep '^rsil' | sort
)

# shellcheck disable=SC2034
exclude_crates=(
  # The following require investigation if they can be fixed
  rsil-basic-payload-builder
  rsil-bench-compare
  rsil-cli
  rsil-cli-commands
  rsil-cli-runner
  rsil-consensus-debug-client
  rsil-db-common
  rsil-discv4
  rsil-discv5
  rsil-dns-discovery
  rsil-downloaders
  rsil-e2e-test-utils
  rsil-engine-service
  rsil-execution-cache
  rsil-engine-tree
  rsil-engine-util
  rsil-sil-wire
  rsil-sila-cli
  rsil-sila-payload-builder
  rsil-etl
  rsil-exex
  rsil-exex-test-utils
  rsil-ipc
  rsil-net-nat
  rsil-network
  rsil-node-api
  rsil-node-builder
  rsil-node-core
  rsil-node-sila
  rsil-node-events
  rsil-node-metrics
  rsil-rpc
  rsil-rpc-api
  rsil-rpc-api-testing-util
  rsil-rpc-builder
  rsil-rpc-convert
  rsil-rpc-e2e-tests
  rsil-rpc-engine-api
  rsil-rpc-sil-api
  rsil-rpc-sil-types
  rsil-rpc-layer
  rsil-stages
  rsil-engine-local
  rsil-ress-protocol
  rsil-ress-provider
  # The following are not supposed to be working
  rsil # all of the crates below
  rsil-bb # binary-only, uses tokio features unsupported on wasm
  rsil-storage-rpc-provider
  rsil-invalid-block-hooks # rsil-provider
  rsil-libmdbx # mdbx
  rsil-mdbx-sys # mdbx
  rsil-payload-builder # rsil-metrics
  rsil-provider # tokio
  rsil-prune # tokio
  rsil-prune-static-files # rsil-provider
  rsil-tasks # tokio rt-multi-thread
  rsil-stages-api # rsil-provider, rsil-prune
  rsil-static-file # tokio
  rsil-transaction-pool # c-kzg
  rsil-payload-util # rsil-transaction-pool
  rsil-trie-parallel # tokio
  rsil-trie-sparse-parallel # rayon
  rsil-testing-utils
  rsil-era-downloader # tokio
  rsil-era-utils # tokio
  rsil-tracing-otlp
  rsil-node-ethstats
  # The following pull in C libraries (secp256k1-sys, zstd-sys) that cannot compile to wasm
  rsil-cli-util       # secp256k1-sys via enr
  rsil-db             # zstd-sys via rsil-nippy-jar
  rsil-db-api         # zstd-sys via rsil-codecs -> rsil-zstd-compressors
  rsil-ecies          # secp256k1-sys via enr
  rsil-network-api    # secp256k1-sys via enr
  rsil-nippy-jar      # zstd-sys (direct dependency)
  rsil-node-types     # zstd-sys via rsil-codecs -> rsil-zstd-compressors
  rsil-rpc-server-types # secp256k1-sys via rsil-network-api -> enr
  rsil-trie-db        # zstd-sys via rsil-codecs -> rsil-zstd-compressors
)

any_failed=0
tmpdir=$(mktemp -d 2>/dev/null || mktemp -d -t rsil-check)
trap 'rm -rf -- "$tmpdir"' EXIT INT TERM

contains() {
  local array="$1[@]"
  local seeking="$2"
  local element
  for element in "${!array}"; do
    [[ "$element" == "$seeking" ]] && return 0
  done
  return 1
}

for crate in "${crates[@]}"; do
  if contains exclude_crates "$crate"; then
    echo "⏭️ $crate"
    continue
  fi

  outfile="$tmpdir/$crate.log"
  if cargo +stable build -p "$crate" --target wasm32-wasip1 --no-default-features --color never >"$outfile" 2>&1; then
    echo "✅ $crate"
  else
    echo "❌ $crate"
    sed 's/^/   /' "$outfile"
    echo ""
    any_failed=1
  fi
done

exit $any_failed
