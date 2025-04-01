# Concurrent Lurk

## Semantics

```js
// This how we would like to write ping and pong.
// function pong(pid) {
//   const msg = receive();
//   if (car(msg) === ping.key) {
//     emit(list('got ping', msg));
//     const other = car(cdr(msg));
//     send(other, list(pong.key, pid));
//   }
//   pong(pid)
// }

// function ping(pid) {
//   const other = spawn(pong);

//   send(other, list(ping.key, pid));
//   const msg = receive();

//   if (car(msg) === pong.key) {
//     emit(list('got pong', msg));
//   }
// }


// This is how we need to write them for now:
function pong(pid) {
  cons([receive.key],
       msg => {
         if (car(msg) === ping.key) {
           emit(list('got ping', msg));
           cons(list(send.key, car(cdr(msg)), list(pong.key, pid)),
                () => pong(pid)
               );
         } else {
           pong(pid)
         }
       })
}
function ping(pid) {
  cons([spawn.key, pong],
       otherPid => {
         cons(list(send.key, otherPid, list(ping.key, pid)),
              () => cons([receive.key],
                         msg => {
                           if (car(msg) === pong.key) {
                             emit(list('got pong', msg));
                           }
                           cons(nil, nil)
                         }))
       })
}
```

Initialize the root chain's state with its (arbitrarily assigned) PID.
```js
const ping1 = ping(PING_ID)
```
Linera side:
```bash
# Create data blob with ping-pong, start a chain with ping.
# Upon starting, the chain needs to take the given state and eat its own PID.
# We should get: `((:spawn pong) . #fun(...)`
```
Then, we need to read the PID of the new chain.
```bash
# TODO
```
We also need to manually start the pong program on another Lurk instance.
```js
const pong1 = pong(PONG_ID)
```
Then, start the pong chain
```bash
# TODO start pong chain
```
Because ping sent a spawn message, it needs to be resumed with the new PID as input.
```js
const ping2 = transition(ping1, PONG_ID)
```
Then, we need to prove this transition on the ping chain.
```bash
# TODO prove transition.
# Since the result is a `:send` control message, also send a message to pong chain.
# On the pong chain side, it should receive the message, and then update the application-level message queue.
```
Therefore, since pong is waiting to receive a message, resume pong with exactly that message:
```js
const ping2 = transition(ping1, PONG_ID)
```



```
LurkScript REPL welcomes you.
lurk-user> load('ping-pong.ls')
Loading ping-pong.ls
t
t

// Initialize the root chain's state with its (arbitrarily assigned) PID.
lurk-user> const ping1 = ping(12345)

[12 iterations] => ((:spawn pong) . #fun((other-pid) ((cons (list :send other-pid (list :ping pid)) (lambda nil (cons '(:receive) (lambda (msg) (if (eq (car msg) :pong) (emit (list "got pong" msg))))))))))

// Check the output part of ping's initial state:
lurk-user> car(ping1)

// It is a control message, requesting a spawn using `pong` as the initial function for the new process/proofchain.
[2 iterations] => (:spawn pong)


// Therefore, initialize the state of a new proofchain by calling pong with its PID.
lurk-user> const pong1 = pong(98765)
[12 iterations] => ((:receive) . #fun((msg) ((begin (emit (list "msg: " msg)) (if (eq (car msg) :ping) (cons (list :send (car (cdr message)) (list :pong pid)) (lambda nil (emit (list "got ping" msg))))) (pong pid)))))

// Check pong's initial output.
lurk-user> car(pong1)

// Output is a receive control message, so pong blocks waiting for input.
[2 iterations] => (:receive)

// Because ping sent a spawn message, it needs to be resumed with the new PID as input.
lurk-user> const ping2 = transition(ping1, 98765)

[19 iterations] => ((:send 98765 (:ping 12345)) . #fun(nil ((cons '(:receive) (lambda (msg) (if (eq (car msg) :pong) (emit (list "got pong" msg))))))))

// Check its new output:
lurk-user> car(ping2)

// It is a send control message targeting pong (PID 98765).
[2 iterations] => (:send 98765 (:ping 12345))

// Therefore, since pong is waiting to receive a message, resume pong with exactly that message:
lurk-user> const pong2 = transition(pong1, car(cdr(cdr(car(ping2)))))

// pong emits a message announcing receipt:
("got ping" (:ping 12345))

[33 iterations] => ((:send 12345 (:pong 98765)) . #fun(nil ((pong pid))))

// Check pong's output.
lurk-user> car(pong2)

// It is a send control message targeting ping (PID 12345)
[2 iterations] => (:send 12345 (:pong 98765))

// Because ping's last response was a non-blocking send, we resume it with no input.
lurk-user> const ping3 = transition(ping2)

[13 iterations] => ((:receive) . #fun((msg) ((begin (if (eq (car msg) :pong) (emit (list "got pong" msg))) (cons nil nil)))))

// Check ping's new output:
lurk-user> car(ping3)

// It is a blocking receive.
[2 iterations] => (:receive)

// Pass ping the message that pong sent.
lurk-user> const ping4 = transition(ping3, car(cdr(cdr(car(pong2)))))

// It emits a message confirming receipt.
("got pong" (:pong 98765))

// And returns a terminal continuation. Ping is done.
[26 iterations] => (nil)

// Resume pong after its send.
lurk-user> const pong3 = transition(pong2)

[10 iterations] => ((:receive) . #fun((msg) ((if (eq (car msg) :ping) (begin (emit (list "got ping" msg)) (cons (list :send (car (cdr msg)) (list :pong pid)) (lambda nil (pong pid)))) (pong pid)))))

// Check its output.
lurk-user> car(pong3)

// It is still ready to receive another message.
[2 iterations] => (:receive)

```

## Usage

### Setting up

Make sure you have the `linera` binary in your `PATH`, and that it is compatible with your
`linera-sdk` version.

For scripting purposes, we also assume that the BASH function `linera_spawn` is defined.
From the root of Linera repository, this can be achieved as follows:

```bash
export PATH="$PWD/target/debug:$PATH"

source /dev/stdin <<<"$(linera net helper 2>/dev/null)"
FAUCET_PORT=8079
FAUCET_URL=http://localhost:$FAUCET_PORT
linera net up --with-faucet --faucet-port $FAUCET_PORT &
LINERA_TMP_DIR=$(mktemp -d)
sleep 1

export LINERA_WALLET_1="$LINERA_TMP_DIR/wallet_1.json"
export LINERA_STORAGE_1="rocksdb:$LINERA_TMP_DIR/client_1.db"
export LINERA_WALLET_2="$LINERA_TMP_DIR/wallet_2.json"
export LINERA_STORAGE_2="rocksdb:$LINERA_TMP_DIR/client_2.db"

linera --with-wallet 1 wallet init --faucet $FAUCET_URL
INFO_1=($(linera --with-wallet 1 wallet request-chain --faucet $FAUCET_URL)) && echo $INFO_1

CHAIN_1="b7a85e90acb4badf7d04a239b2b6721bac885c3422cf3b93861695f1a5a33d9e"
OWNER_1=$(linera -w1 keygen) && echo $OWNER_1

APP_ID=$(linera -w1 --wait-for-outgoing-messages \
  project publish-and-create examples/concurrent-lurk concurrent_lurk $CHAIN_1)

PING_GENESIS_ID=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/6231c57981bd63e6c2cf0021a5fb1f69727247d4006d0ffb70bb0a45851119/genesis_state $CHAIN_1)

TRANSITION_PING_1=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/6231c57981bd63e6c2cf0021a5fb1f69727247d4006d0ffb70bb0a45851119/_0 $CHAIN_1)

TRANSITION_PING_2=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/6231c57981bd63e6c2cf0021a5fb1f69727247d4006d0ffb70bb0a45851119/_2 $CHAIN_1)

TRANSITION_PING_3=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/6231c57981bd63e6c2cf0021a5fb1f69727247d4006d0ffb70bb0a45851119/_3 $CHAIN_1)

linera -w1 service --port 8080 &
sleep 1
```

Start the local Linera network and run a faucet:

```bash

```

Create the user wallets and add chains to them:

```bash
export LINERA_WALLET_1="$LINERA_TMP_DIR/wallet_1.json"
export LINERA_STORAGE_1="rocksdb:$LINERA_TMP_DIR/client_1.db"
export LINERA_WALLET_2="$LINERA_TMP_DIR/wallet_2.json"
export LINERA_STORAGE_2="rocksdb:$LINERA_TMP_DIR/client_2.db"

linera --with-wallet 1 wallet init --faucet $FAUCET_URL

INFO_1=($(linera --with-wallet 1 wallet request-chain --faucet $FAUCET_URL)) && echo $INFO_1

CHAIN_1="${INFO_1[0]}"
OWNER_1="${INFO_1[3]}"
```

Note that `linera --with-wallet 1` or `linera -w1` is equivalent to `linera --wallet
"$LINERA_WALLET_1" --storage "$LINERA_STORAGE_1"`.

### Creating the Lurk Microchain

```bash
APP_ID=$(linera -w1 --wait-for-outgoing-messages \
  project publish-and-create examples/concurrent-lurk concurrent_lurk $CHAIN_1)

PING_GENESIS_ID=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/6231c57981bd63e6c2cf0021a5fb1f69727247d4006d0ffb70bb0a45851119/genesis_state $CHAIN_1)

TRANSITION_PING_1=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/6231c57981bd63e6c2cf0021a5fb1f69727247d4006d0ffb70bb0a45851119/_0 $CHAIN_1)

OWNER_1=$(linera -w1 keygen)

linera -w1 service --port 8080 &
sleep 1
```

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

MESSAGE_ID="b7a85e90acb4badf7d04a239b2b6721bac885c3422cf3b93861695f1a5a33d9e050000000000000000000000"
CHAIN_2="a98dd4332a0383d36bf0131c985c0cb8d45448bc91841bf0737b14d139b3f595"

linera -w1 assign --owner $OWNER_1 --message-id $MESSAGE_ID

PONG_GENESIS_ID=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/8c09b93fe69344a90562faeb7f144c63476f2c93301dee845c0e5342500949/genesis_state $CHAIN_2)

TRANSITION_PONG_1=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/8c09b93fe69344a90562faeb7f144c63476f2c93301dee845c0e5342500949/_1 $CHAIN_2)

TRANSITION_PONG_2=$(linera -w1 publish-data-blob \
  ~/.lurk/microchains/8c09b93fe69344a90562faeb7f144c63476f2c93301dee845c0e5342500949/_4 $CHAIN_2)

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

```gql,uri=http://localhost:8081/chains/$MICROCHAIN/applications/$APP_ID
mutation { transition(
  chainProof: \"$TRANSITION_PING_1\"
) }

mutation { transition(
  chainProof: \"$TRANSITION_PONG_1\"
) }

mutation { transition(
  chainProof: \"$TRANSITION_PING_2\"
) }
```
