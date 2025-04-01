# How to run the E2E test

Open four terminals. In the first, start the linera server:

```bash
FAUCET_PORT=8079
FAUCET_URL=http://localhost:$FAUCET_PORT
linera --send-timeout-ms 50000 --recv-timeout-ms 50000 net up --with-faucet --faucet-port $FAUCET_PORT &
```

In the second, run the benchmark:

```bash
LINERA_TMP_DIR=$(mktemp -d) && echo $LINERA_TMP_DIR

FAUCET_PORT=8079
FAUCET_URL=http://localhost:$FAUCET_PORT
export LINERA_WALLET_1="$LINERA_TMP_DIR/wallet_1.json"
export LINERA_STORAGE_1="rocksdb:$LINERA_TMP_DIR/client_1.db"

linera --with-wallet 1 wallet init --faucet $FAUCET_URL
INFO=($(linera --with-wallet 1 wallet request-chain --faucet $FAUCET_URL))

export PING_CHAIN=$(echo $INFO | awk '{print $1}')
export OWNER=$(linera -w1 keygen)

lurk load examples/concurrent-lurk/ping-pong-test.lurk --linera --with-wallet 1
```

In the 3rd terminal:
```bash
LINERA_TMP_DIR=<whatever it was assigned to in the 2nd terminal>

FAUCET_PORT=8079
FAUCET_URL=http://localhost:$FAUCET_PORT
export LINERA_WALLET_2="$LINERA_TMP_DIR/wallet_2.json"
export LINERA_STORAGE_2="rocksdb:$LINERA_TMP_DIR/client_2.db"
linera --with-wallet 2 wallet init --faucet $FAUCET_URL
INFO=($(linera --with-wallet 2 wallet request-chain --faucet $FAUCET_URL)) && echo $INFO

export PING_CHAIN=$(echo $INFO | awk '{print $1}')
export OWNER=$(linera -w2 keygen)

lurk load examples/concurrent-lurk/ping-pong-test.lurk --linera --with-wallet 2
```

In the 4rd terminal:
```bash
LINERA_TMP_DIR=<whatever it was assigned to in the 2nd terminal>

FAUCET_PORT=8079
FAUCET_URL=http://localhost:$FAUCET_PORT
export LINERA_WALLET_3="$LINERA_TMP_DIR/wallet_3.json"
export LINERA_STORAGE_3="rocksdb:$LINERA_TMP_DIR/client_3.db"

linera --with-wallet 3 wallet init --faucet $FAUCET_URL
INFO=($(linera --with-wallet 3 wallet request-chain --faucet $FAUCET_URL)) && echo $INFO

export PING_CHAIN=$(echo $INFO | awk '{print $1}')
export OWNER=$(linera -w3 keygen)

lurk load examples/concurrent-lurk/ping-pong-test.lurk --linera --with-wallet 3
```