#!/bin/bash
# Script to automate E2E test for Linera with concurrent Lurk
# Runs clients in background without tmux
# Only times the duration of lurk loading

# Default number of clients to run
NUM_CLIENTS=${1:-3}

# Create a temporary directory for wallets and storage
LINERA_TMP_DIR=$(mktemp -d)
echo "Using temporary directory: $LINERA_TMP_DIR"

# Create timing log file
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
mkdir "./examples/concurrent-lurk/run_e2e_${TIMESTAMP}"
LOG_DIR="./examples/concurrent-lurk/run_e2e_${TIMESTAMP}"

SERVER_LOG="$LOG_DIR/server.log"

TIMING_LOG="$LOG_DIR/lurk_load_times.txt"
echo "# Lurk Load Times (seconds)" > $TIMING_LOG
echo "# Client_ID | Start_Time | End_Time | Duration" >> $TIMING_LOG

# Create a progress tracking file
PROGRESS_FILE="$LOG_DIR/progress.txt"
echo "0" > $PROGRESS_FILE  # Initialize with 0 completed clients

# Start the linera server if it's not already running
if ! nc -z localhost 8079 &>/dev/null; then
    echo "Starting Linera server with faucet..."
    FAUCET_PORT=8079
    FAUCET_URL=http://localhost:$FAUCET_PORT
    linera --send-timeout-ms 50000 --recv-timeout-ms 50000 net up --with-faucet --faucet-port $FAUCET_PORT > $SERVER_LOG 2>&1 &
    SERVER_PID=$!
    echo "Linera server started with PID: $SERVER_PID"
    
    # Wait for server to be ready
    for i in {1..30}; do
        if nc -z localhost 8079 &>/dev/null; then
            echo "Server is ready!"
            break
        fi
        echo "Waiting for server to start... ($i/30)"
        sleep 1
    done
else
    echo "Linera server is already running on port 8079"
fi

FAUCET_PORT=8079
FAUCET_URL=http://localhost:$FAUCET_PORT

# Function to update progress
update_progress() {
    local current=$(cat $PROGRESS_FILE)
    echo $((current + 1)) > $PROGRESS_FILE
}

# Function to run a client in the background
run_client() {
    local client_id=$1
    local client_log="$LOG_DIR/client_${client_id}.log"
    
    # Redirect all output to log file
    {
        # Basic client setup
        echo "# Client $client_id starting at $(date)"
        export PORT=808${client_id}
        export LINERA_WALLET_${client_id}="$LINERA_TMP_DIR/wallet_${client_id}.json"
        export LINERA_STORAGE_${client_id}="rocksdb:$LINERA_TMP_DIR/client_${client_id}.db"
        
        echo "Initializing wallet..."
        linera --with-wallet ${client_id} wallet init --faucet $FAUCET_URL
        
        echo "Requesting chain..."
        INFO=($(linera --with-wallet ${client_id} wallet request-chain --faucet $FAUCET_URL))
        
        echo "Setting up environment..."
        export PING_CHAIN=$(echo $INFO | awk '{print $1}')
        export OWNER=$(linera -w${client_id} keygen)
        
        # Time lurk loading - only part we care about timing
        echo "Starting lurk load for client $client_id..."
        START_TIME=$(date +%s.%N)
        RUST_BACKTRACE=1 LOG_DIR="$LOG_DIR" lurk load examples/concurrent-lurk/ping-pong-test.lurk --linera --with-wallet ${client_id}
        END_TIME=$(date +%s.%N)
        DURATION=$(echo "$END_TIME - $START_TIME" | bc)
        echo "$client_id | $START_TIME | $END_TIME | $DURATION" >> $TIMING_LOG
        echo "Client $client_id lurk load took $DURATION seconds"
        
        # Update progress counter
        update_progress
        
        echo "# Client $client_id completed at $(date)"
    } &> $client_log
}

# Run all clients in parallel
echo "Starting $NUM_CLIENTS clients in parallel..."
for ((i=1; i<=NUM_CLIENTS; i++)); do
    run_client $i &
    PIDS[$i]=$!
    echo "Started client $i with PID ${PIDS[$i]}"
    
    # Add a small delay between client starts to reduce contention
    sleep 0.5
done

# Wait for all clients to complete while showing progress
echo "Waiting for all clients to complete..."
echo "Press Ctrl+C to stop (this won't kill the running clients)"
while true; do
    completed=$(cat $PROGRESS_FILE)
    
    if [ $completed -eq $NUM_CLIENTS ]; then
        echo -e "\nAll clients have completed!"
        break
    fi
    
    sleep 1
done

# Calculate and display statistics
echo "" >> $TIMING_LOG
echo "Statistics:" >> $TIMING_LOG
echo "-----------" >> $TIMING_LOG
TIMES=$(awk -F' \\| ' 'NR>2 {print $4}' $TIMING_LOG)
if [ -n "$TIMES" ]; then
    # Calculate min, max, average
    MIN=$(echo "$TIMES" | sort -n | head -1)
    MAX=$(echo "$TIMES" | sort -n | tail -1)
    AVG=$(echo "$TIMES" | awk '{sum+=$1} END {print sum/NR}')
    
    echo "Minimum time: $MIN seconds" >> $TIMING_LOG
    echo "Maximum time: $MAX seconds" >> $TIMING_LOG
    echo "Average time: $AVG seconds" >> $TIMING_LOG
fi

cargo run --bin parse_logs $LOG_DIR/server.log $LOG_DIR/server_benchmarks.md 0

# Collect and display results
echo ""
echo "Test completed! Results:"
echo "------------------------"
cat $TIMING_LOG

echo ""
echo "For full client logs, check:"
for ((i=1; i<=NUM_CLIENTS; i++)); do
    echo "  Client $i: $LOG_DIR/client_${i}.log"
done

echo ""
echo "To clean up after testing, run:"
echo "  kill $SERVER_PID  # Kill the server if you started it"
echo "  rm -rf $LINERA_TMP_DIR  # Remove the temporary directory"