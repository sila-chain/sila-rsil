# rsil-bb

A modified rsil node for benchmarking **big block** execution — payloads that merge transactions from multiple consecutive blocks into a single block to simulate high-gas workloads.

> **Not for production use.** rsil-bb disables some consensus-related validations to allow artificially large blocks. It is intended solely for performance benchmarking.

## How it works

rsil-bb extends the standard Sila node with:

1. **Multi-segment execution** — a custom `rsil_newPayload` handler that accepts optional `BigBlockData` alongside the payload. When present, the block is executed in multiple segments, each with its own SAVM environment (matching the original blocks that were merged).

2. **Relaxed consensus** — the gas-limit bound-divisor check and blob gas validation are skipped, since merged blocks exceed single-block limits.

## Quick start

The full workflow has five steps: **build** rsil-bb, **install** txgen tools, **extract** big-block payloads, **start** rsil-bb, and **replay** the payloads.

### Prerequisites

- A synced rsil datadir for the target chain (e.g. hoodi)
- Rust toolchain
- An archive RPC endpoint that supports `debug_getRawBlock`

### 1. Build rsil-bb

```bash
cargo build --profile profiling -p rsil-bb
```

### 2. Install txgen tools

Install `txgen-sila` for big-block extraction and `bench` for Engine API replay from [tempoxyz/txgen](https://github.com/tempoxyz/txgen):

```bash
cargo install --git https://github.com/tempoxyz/txgen txgen-sila --locked
cargo install --git https://github.com/tempoxyz/txgen bench-cli --locked
```

### 3. Extract big-block payloads

Merge source blocks into synthetic big-block payloads, starting at the block following the one the node is currently synced to (i.e. the next block the node would process):

```bash
txgen-sila extract-big-blocks \
    --rpc https://rpc.hoodi.ethpandaops.io \
    --from 910020 \
    --count 5 \
    --target-gas 2G \
    --output /tmp/big-blocks.ndjson
```

This produces an NDJSON file containing one rsil-bb-compatible big-block payload per line. `--target-gas` accepts bare units or `K`, `M`, `G` suffixes.

### 4. Start rsil-bb

```bash
rsil-bb node \
    --datadir /data/rsil/hoodi \
    --chain hoodi \
    --http --http.api debug,sil \
    --authrpc.jwtsecret /tmp/jwt.hex \
    -d
```

### 5. Replay payloads

```bash
bench send-blocks \
    --engine http://localhost:8551 \
    --jwt-secret /tmp/jwt.hex \
    --input /tmp/big-blocks.ndjson \
    --wait-for-persistence every:2 \
    --report json:/tmp/rsil-bb-report.json
```

`bench send-blocks` submits each payload through rsil's custom `rsil_newPayload` and `rsil_forkchoiceUpdated` Engine API methods.
