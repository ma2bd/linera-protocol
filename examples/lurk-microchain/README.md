# Lurk Microchain

## Usage

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

Create the user wallets and add chains to them:

```bash
export LINERA_WALLET_1="$LINERA_TMP_DIR/wallet_1.json"
export LINERA_STORAGE_1="rocksdb:$LINERA_TMP_DIR/client_1.db"
export LINERA_WALLET_2="$LINERA_TMP_DIR/wallet_2.json"
export LINERA_STORAGE_2="rocksdb:$LINERA_TMP_DIR/client_2.db"

linera --with-wallet 1 wallet init --faucet $FAUCET_URL
linera --with-wallet 2 wallet init --faucet $FAUCET_URL

INFO_1=($(linera --with-wallet 1 wallet request-chain --faucet $FAUCET_URL))
INFO_2=($(linera --with-wallet 2 wallet request-chain --faucet $FAUCET_URL))
CHAIN_1="${INFO_1[0]}"
CHAIN_2="${INFO_2[0]}"
OWNER_1="${INFO_1[3]}"
OWNER_2="${INFO_2[3]}"
```

Note that `linera --with-wallet 1` or `linera -w1` is equivalent to `linera --wallet
"$LINERA_WALLET_1" --storage "$LINERA_STORAGE_1"`.

### Creating the Lurk Microchain

```bash
APP_ID=$(linera -w1 --wait-for-outgoing-messages \
  project publish-and-create examples/lurk-microchain lurk_microchain $CHAIN_1)

GENESIS_BLOB_ID=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/5e5eca21f5e9fe4967e15e99078d0f86248239db3471b1c63197f4df7cc162/genesis_state)

TRANSITION_0=$(linera -w2 publish-data-blob \
  ~/.lurk/microchains/5e5eca21f5e9fe4967e15e99078d0f86248239db3471b1c63197f4df7cc162/_0)

OWNER_1=$(linera -w1 keygen)
OWNER_2=$(linera -w2 keygen)

linera -w1 service --port 8080 &
sleep 1
```

Type each of these in the GraphiQL interface and substitute the env variables with their actual values that we've defined above.

The `start` mutation starts a new game. We specify the two players using their new public keys,
on the URL you get by running `echo "http://localhost:8080/chains/$CHAIN_1/applications/$APP_ID"`:

```gql,uri=http://localhost:8080/chains/$CHAIN_1/applications/$APP_ID
mutation {
  start(
    accounts: [
        \"$OWNER_1\",
        \"$OWNER_2\"
    ],
    chainState: \"$GENESIS_BLOB_ID\"
  )
}
```

The app's main chain keeps track of the games in progress, by public key:

```gql,uri=http://localhost:8080/chains/$CHAIN_1/applications/$APP_ID
query {
  chains {
    keys(count: 3)
  }
}
```

It contains the temporary chain's ID, and the ID of the message that created it:

```gql,uri=http://localhost:8080/chains/$CHAIN_1/applications/$APP_ID
query {
  chains {
    entry(key: \"$OWNER_1\") {
      value {
        messageId chainId
      }
    }
  }
}
```

Set the `QUERY_RESULT` variable to have the result returned by the previous query, and `HEX_CHAIN` and `MESSAGE_ID` will be properly set for you.
Alternatively you can set the variables to the `chainId` and `messageId` values, respectively, returned by the previous query yourself.
Using the message ID, we can assign the new chain to the key in each wallet:

```bash
kill %% && sleep 1    # Kill the service so we can use CLI commands for wallet 0.

MICROCHAIN=a416d5da467baed501ab83591d601c8db824d25c9ce790ead7e1e27e9949f4c1
MESSAGE_ID=b7a85e90acb4badf7d04a239b2b6721bac885c3422cf3b93861695f1a5a33d9e060000000000000000000000

linera -w1 assign --owner $OWNER_1 --message-id $MESSAGE_ID
linera -w2 assign --owner $OWNER_2 --message-id $MESSAGE_ID

linera -w1 service --port 8080 &
linera -w2 service --port 8081 &
sleep 1
```

### Interacting with the Lurk microchain

Now the first player can make a move by navigating to the URL you get by running `echo "http://localhost:8080/chains/$MICROCHAIN/applications/$APP_ID"`:

```bash
TRANSITION_0=$(linera -w2 publish-data-blob \
  ~/.lurk/microchains/5e5eca21f5e9fe4967e15e99078d0f86248239db3471b1c63197f4df7cc162/_0 $MICROCHAIN)
```

```gql,uri=http://localhost:8080/chains/$MICROCHAIN/applications/$APP_ID
mutation { transition(chainProof: \"$TRANSITION_0\") }
```

And the second player player at the URL you get by running `echo "http://localhost:8081/chains/$MICROCHAIN/applications/$APP_ID"`:

```gql,uri=http://localhost:8081/chains/$MICROCHAIN/applications/$APP_ID
mutation { transition(
  chainProof: "$TRANSITION_0"
) }
```
