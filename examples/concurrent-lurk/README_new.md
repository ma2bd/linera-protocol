# Concurrent Lurk

## Usage

### Installing Lurk

Before you begin, make sure you have the `lurk` binary installed.

In a separate directory of your choosing, do the following:

```bash
git checkout git@github.com:lurk-lab/lurk.git && cd lurk
git checkout whz/on-linera
cargo install --locked --path .
```

You can test your installation by trying to run the `lurk` command, which should open the Lurk REPL:

```bash
commit: 2025-03-24 3d4bb68f4618a9adf487e2e4d0fbb905ca26c1e3
Lurk REPL welcomes you.
lurk-user> 
```

For more information, go to https://github.com/lurk-lab/lurk or the [Lurk User Manual](https://docs.argument.xyz/).

### Setting up

Make sure you have the `linera` binary in your `PATH`, and that it is compatible with your
`linera-sdk` version.

For scripting purposes, we also assume that the BASH function `linera_spawn` is defined.
From the root of Linera repository, this can be achieved as follows:

```bash
export PATH="$PWD/target/debug:$PATH"
source /dev/stdin <<<"$(linera net helper 2>/dev/null)"
```

Start the local Linera network and run a faucet:

```bash
FAUCET_PORT=8079
FAUCET_URL=http://localhost:$FAUCET_PORT
linera net up --with-faucet --faucet-port $FAUCET_PORT &
LINERA_TMP_DIR=$(mktemp -d)
```

Create the user wallet and add chains to it:

```bash
export LINERA_WALLET_1="$LINERA_TMP_DIR/wallet_1.json"
export LINERA_STORAGE_1="rocksdb:$LINERA_TMP_DIR/client_1.db"

linera --with-wallet 1 wallet init --faucet $FAUCET_URL

INFO=($(linera --with-wallet 1 wallet request-chain --faucet $FAUCET_URL)) && echo $INFO
export PING_CHAIN="${INFO[0]}"
export OWNER=$(linera -w1 keygen)
```

Note that `linera --with-wallet 1` or `linera -w1` is equivalent to `linera --wallet
"$LINERA_WALLET_1" --storage "$LINERA_STORAGE_1"`.

### Running the Concurrent Lurk program

Now let's demonstrate how to run a Concurrent Lurk program. 
Run the `lurk --linera --with-wallet 1` command, and type in the following commands into the REPL:

```bash
!(load "examples/concurrent-lurk/ping-pong.lurk")

!(def contract "examples/target/wasm32-unknown-unknown/release/concurrent_lurk_contract.wasm")
!(def service "examples/target/wasm32-unknown-unknown/release/concurrent_lurk_service.wasm")

!(def port "8081")
!(def ping-chain-id !(env-var "PING_CHAIN"))
!(def owner !(env-var "OWNER"))

!(defq app-id !(linera-start ping-chain-id contract service))
```

This publishes a Concurrent Lurk application on Linera in the background. There will be some errors, but you can ignore them (Linera over-prints them for some reason).
Note that we've used Linera's default chain, `$PING_CHAIN`, as the genesis process, and that the the value of the application id has been bound to `app-id`.
Next, let's initialize the process with a state:

```bash
!(def ping1 (ping ping-chain-id))
!(microchain-start port ping-chain-id app-id owner ping1)
```

Upon sucess, the REPL will print out a message: `Process spawned a new chain: 83990e573e43c72806fe93036de3418b9d3e57108e1cf4dabb6b5893b3f1a3b2`. That is our pong chain id.
```bash
!(def pong-chain-id "83990e573e43c72806fe93036de3418b9d3e57108e1cf4dabb6b5893b3f1a3b2")
```

You can check that the state on the ping chain as been initialized to the correct `:spawn` control message, as follows:
```bash
!(microchain-get-state port ping-chain-id app-id)
((:spawn pong) . <Fun ...>)
```

Therefore, we next initialize the state of our new pong chain by calling pong with its PID.

```bash
!(def pong1 (pong pong-chain-id))
!(microchain-start port pong-chain-id app-id owner pong1)
```

The output is a receive control message, so pong blocks waiting for input.

Because ping sent a spawn message, it needs to be resumed with the spawned PID as input.

```bash
!(defq ping2 !(microchain-transition port ping-chain-id app-id ping1 pong-chain-id))
```

It is a send control message targeting pong.

Note that if we pass in the incorrect PID, e.g. `ping-chain-id`, Linera will respond with an error. 
Thus the semantics of Concurrent Lurk are preserved on the Linera side.
```bash
!(defq ping2 !(microchain-transition port ping-chain-id app-id ping1 ping-chain-id))
[11 iterations] => ...
...
2025-03-27T05:50:46.021821Z ERROR linera_execution::wasm::runtime_api: panicked at concurrent-lurk/src/contract.rs:111:17:
assertion `left == right` failed: Incorrect spawn PID.
```

Therefore, since pong is waiting to receive a message, resume pong with exactly that message:

```bash
!(defq pong2 !(microchain-transition port pong-chain-id app-id pong1 (car (cdr (cdr (car ping2))))))
```

Pong emits a message announcing receipt: `("got ping" (:ping "b7a85e90acb4badf7d04a239b2b6721bac885c3422cf3b93861695f1a5a33d9e"))`.
Furthermore, its output is a send control message targeting ping.

Because ping's last response was a non-blocking send, we resume it with no input.

```bash
!(defq ping3 !(microchain-transition port ping-chain-id app-id ping2))
```

ping's new output:


Type each of these in the GraphiQL interface and substitute the env variables with their actual values that we've defined above.

The `start` mutation starts a new game. We specify the two players using their new public keys,
on the URL you get by running `echo "http://localhost:8080/chains/$CHAIN_1/applications/$APP_ID"`:

```gql,uri=http://localhost:8080/chains/$CHAIN_1/applications/$APP_ID
mutation {
  start(
    owner: \"$OWNER_1\"
    chainState: \"$PING_GENESIS_ID\"
  )
}
```

It contains the temporary chain's ID, and the ID of the message that created it:

```gql,uri=http://localhost:8080/chains/$CHAIN_1/applications/$APP_ID
query {
  ready {
    messageId chainId
  }
}
```

Set the `QUERY_RESULT` variable to have the result returned by the previous query, and `HEX_CHAIN` and `MESSAGE_ID` will be properly set for you.
Alternatively you can set the variables to the `chainId` and `messageId` values, respectively, returned by the previous query yourself.
Using the message ID, we can assign the new chain to the key in each wallet:

```bash
kill %% && sleep 1    # Kill the service so we can use CLI commands for wallet 0.

MESSAGE_ID="b7a85e90acb4badf7d04a239b2b6721bac885c3422cf3b93861695f1a5a33d9e040000000000000000000000"
CHAIN_2="0db0ca3358f233ac25c3ecbcb1e9eaa86fa3f520fd1345155ec496973525ece2"

linera -w1 assign --owner $OWNER_1 --message-id $MESSAGE_ID

PONG_GENESIS_ID=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/8c09b93fe69344a90562faeb7f144c63476f2c93301dee845c0e5342500949/genesis_state $CHAIN_2)

TRANSITION_PONG_1=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/8c09b93fe69344a90562faeb7f144c63476f2c93301dee845c0e5342500949/_1 $CHAIN_1)

linera -w1 service --port 8080 &
sleep 1
```

### Interacting with the Lurk microchain

Now the first player can make a move by navigating to the URL you get by running `echo "http://localhost:8080/chains/$CHAIN_2/applications/$APP_ID"`:

```gql,uri=http://localhost:8080/chains/$CHAIN_2/applications/$APP_ID
mutation {
  start(
    owner: \"$OWNER_1\"
    chainState: \"$PONG_GENESIS_ID\"
  )
}
```

And the second player player at the URL you get by running `echo "http://localhost:8081/chains/$MICROCHAIN/applications/$APP_ID"`:

```gql,uri=http://localhost:8081/chains/$MICROCHAIN/applications/$APP_ID
mutation { transition(
  chainProof: "$TRANSITION_0"
) }
```
