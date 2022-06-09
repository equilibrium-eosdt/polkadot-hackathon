# Runtime Distribution Optimization.

## Runtime:

There are 4 modules (pallets) presented:

* asset pallet - stores a set of predefined assets;
* oracle pallet - stores asset prices and sets a new random positive price in range [1.0, 2.0] for each asset every 5 blocks;
* balances pallet - stores balances for each (account, asset) pair.
* distribution pallet - anyone can deposit or withdraw tokens to/from this pallet:
  * `issue` - store assets on dedicated `distribution` account.
  * `deposit` - transfers funds to treasury and updates corresponding entry in `Deposits` storage.
  * `withdraw` - do the opposit thing.
  
  It also has an on-initialize hook that redistributes issued assets between depositors in proportion to their deposit in $USD.

Runtime block duration is reduced to 2 secs.

Pallets are in lack of benchmarks and extrinsics weights since we are insterested in raw performance and substrate won't submit too heavy calls.

## Task:

You have been provided with the simplest implementation of distribution logic, but this implementation has a performance limitation - the more accounts have deposited funds to the distributor pallet the slower the actual distribution among the end users will happen (N users each need to receive their share of M assets). Your task is to make changes to the pallets and/or the substrate framework to speed up this solution.

## How will the solution be tested?

A chain is launched (see [Run multiple validators](`###run-multiple-validators`) + [Run test case](###run-test-case)) with 2 validators (Alice, Bob) and N clients, each having balances in each of the M assets.

One iteration of testing consists of the following steps:
* Each client deposit random amount to distribution;
* Random amount in range [5, 10] of each token issued in distribution and then redistributed;
* Each client withdraw his funds;
* Balances on chain are checked for correct amount after redistribution.

If the data on the chain after several iterations is correct and the block time of the chain has not exceeded 2 sec then N is increasing.

The solution that pass 5 iterations at the maximum N*M wins.

## Run test.

### Run single validator:
```sh
cargo run --release -- --dev
```

### Run multiple validators:
* Alice:
```sh
./target/release/hack-a-node purge-chain --chain local --base-path ./alice -y
./target/release/hack-a-node \
    --chain local \
    --alice --validator --base-path ./alice \
    --port 30333 --ws-port 9944 --rpc-port 9933 \
    --node-key 0000000000000000000000000000000000000000000000000000000000000001
```
* Bob:
```sh
./target/release/hack-a-node purge-chain --chain local --base-path ./bob -y
./target/release/hack-a-node \
    --chain local \
    --bob --validator --base-path ./bob \
    --port 30334 --ws-port 9945 --rpc-port 9934 \
    --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

### Run test case:
```
cd ./test-case
yarn
yarn start -N 100 -M 20 | npx pino-pretty
```