#!/usr/bin/env bash
set -uxo pipefail

crates_to_check=(
    rsil-network-peers
    rsil-trie-common
    rsil-trie-sparse
    rsil-chainspec
    rsil-consensus
    rsil-consensus-common
    rsil-prune-types
    rsil-static-file-types
    rsil-storage-errors
    rsil-execution-errors
    rsil-errors
    rsil-execution-types
    rsil-db-models
    rsil-savm
    rsil-revm
    rsil-storage-api

    ## sila
    rsil-savm-sila
    rsil-sila-forks
    rsil-sila-primitives
    rsil-sila-consensus
)

any_failed=0
tmpdir=$(mktemp -d 2>/dev/null || mktemp -d -t rsil-check)
trap 'rm -rf -- "$tmpdir"' EXIT INT TERM

for crate in "${crates_to_check[@]}"; do
  outfile="$tmpdir/$crate.log"
  if cargo +stable build -p "$crate" --target riscv32imac-unknown-none-elf --no-default-features --color never >"$outfile" 2>&1; then
    echo "✅ $crate"
  else
    echo "❌ $crate"
    sed 's/^/   /' "$outfile"
    echo ""
    any_failed=1
  fi
done

exit $any_failed
