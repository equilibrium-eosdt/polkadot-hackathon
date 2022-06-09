# Runtime Distribution Optimization.

## Inputs:

There are 4 modules (pallets) presented: 

* asset pallet that stores a set of predefined assets (M assets in total)
* oracle pallet, which (with the help of the off-chain worker) sets a new random positive price for each asset every 20 blocks.
* balances pallet, which stores balances for each account in each asset.
* distributor pallet, anyone can deposit or withdraw tokens to/from this pallet. This pallet has a storage which maps accounts to balances in corresponding assets. Every 5 blocks the random amount of tokens in each of the M assets is funneled to the distributor pallet, which has to re-distribute them further pro-rata user balances.

## Task:

You have been provided with the simplest implementation of this logic, but this implementation has a performance limitation - the more accounts have deposited funds to the distributor pallet the slower the actual distribution among the end users will happen (N users each need to receive their share of M assets). Your task is to make changes to the pallets and/or the substrate framework to speed up this solution.

## How will the solution be tested?

A chain is launched with 2 validators (Alice, Bob) and N clients, each having balances in each of the M assets.

One run is consisted of the following steps:
* Each client deposit random amount to distribution;
* 10 tokens issued in distribution and then redistributed;
* Each client withdraw his funds.

If the data on the chain after several runs is correct and the block time of the chain has not exceeded 6 sec then N is increasing.

The solution that works at the maximum N wins.

# Test node

## Run single validator
```sh
cargo run --release -- --dev
```

## Run multiple validators
Alice:
```sh
./target/release/hack-a-node purge-chain --chain local --base-path ./alice -y
./target/release/hack-a-node \
    --chain local \
    --alice --validator --base-path ./alice \
    --port 30333 \
    --ws-port 9944 \
    --rpc-port 9933 \
    --node-key 0000000000000000000000000000000000000000000000000000000000000001
```

Bob:
```sh
./target/release/hack-a-node purge-chain --chain local --base-path ./bob -y
./target/release/hack-a-node \
    --chain local \
    --bob --validator --base-path ./bob \
    --port 30334 \
    --ws-port 9945 \
    --rpc-port 9934 \
    --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

## Run test case
```
cd ./test-case
yarn
yarn start -N 100 -M 20 | npx pino-pretty
```