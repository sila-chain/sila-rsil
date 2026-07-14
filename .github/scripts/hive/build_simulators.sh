#!/usr/bin/env bash
set -eo pipefail

fixture_variant="${1:-osaka}"

case "${fixture_variant}" in
    amsterdam)
        eels_fixtures="https://github.com/sila-chain/execution-spec-tests/releases/download/bal@v7.1.1/fixtures_bal.tar.gz"
        eels_branch="devnets/bal/7"
        ;;
    osaka)
        eels_fixtures="https://github.com/sila-chain/execution-spec-tests/releases/download/v5.3.0/fixtures_develop.tar.gz"
        eels_branch="sila-mainnet"
        ;;
    *)
        echo "unknown hive fixture variant: ${fixture_variant}"
        exit 1
        ;;
esac

# Create the hive_assets directory
mkdir hive_assets/

cd hivetests
go build .

./hive -client rsil # first builds and caches the client

# Run each hive command in the background for each simulator and wait
echo "Building images"
./hive -client rsil --sim "sila/eels/consume-engine" \
    --sim.buildarg fixtures="${eels_fixtures}" \
    --sim.buildarg branch="${eels_branch}" \
    --sim.timelimit 1s || true &
./hive -client rsil --sim "sila/eels/consume-rlp" \
    --sim.buildarg fixtures="${eels_fixtures}" \
    --sim.buildarg branch="${eels_branch}" \
    --sim.timelimit 1s || true &
./hive -client rsil --sim "sila/engine" -sim.timelimit 1s || true &
./hive -client rsil --sim "devp2p" -sim.timelimit 1s || true &
./hive -client rsil --sim "sila/rpc-compat" -sim.timelimit 1s || true &
./hive -client rsil --sim "smoke/genesis" -sim.timelimit 1s || true &
./hive -client rsil --sim "smoke/network" -sim.timelimit 1s || true &
./hive -client rsil --sim "sila/sync" -sim.timelimit 1s || true &
wait

# Run docker save in parallel, wait and exit on error
echo "Saving images"
saving_pids=( )
docker save hive/hiveproxy:latest -o ../hive_assets/hiveproxy.tar & saving_pids+=( $! )
docker save hive/simulators/devp2p:latest -o ../hive_assets/devp2p.tar & saving_pids+=( $! )
docker save hive/simulators/sila/engine:latest -o ../hive_assets/engine.tar & saving_pids+=( $! )
docker save hive/simulators/sila/rpc-compat:latest -o ../hive_assets/rpc_compat.tar & saving_pids+=( $! )
docker save hive/simulators/sila/eels/consume-engine:latest -o ../hive_assets/eels_engine.tar & saving_pids+=( $! )
docker save hive/simulators/sila/eels/consume-rlp:latest -o ../hive_assets/eels_rlp.tar & saving_pids+=( $! )
docker save hive/simulators/smoke/genesis:latest -o ../hive_assets/smoke_genesis.tar & saving_pids+=( $! )
docker save hive/simulators/smoke/network:latest -o ../hive_assets/smoke_network.tar & saving_pids+=( $! )
docker save hive/simulators/sila/sync:latest -o ../hive_assets/sila_sync.tar & saving_pids+=( $! )
for pid in "${saving_pids[@]}"; do
    wait "$pid" || exit
done

# Make sure we don't rebuild images on the CI jobs
git apply ../.github/scripts/hive/no_sim_build.diff
go build .
mv ./hive ../hive_assets/
