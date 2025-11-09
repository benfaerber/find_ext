#!/bin/bash

# Simple benchmark script for find_ext
# Tests performance with and without caching

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
export FIND_EXT_SEARCH_EXTENSIONS="rs,py,ml,php,kt,ts,js,lua,res,c,cpp,hs,hc"
export FIND_EXT_DISALLOWED_FOLDERS="node_modules,target/debug/build,target/release/build"
export FIND_EXT_CACHE_FILE="/tmp/find_ext_benchmark_cache.csv"

BINARY="${1:-./target/release/find_ext}"
BENCHMARK_DIR="${2:-.}"
ITERATIONS="${3:-10}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   find_ext Benchmark${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Binary: $BINARY"
echo "Directory: $BENCHMARK_DIR"
echo "Iterations: $ITERATIONS"
echo ""

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${YELLOW}Binary not found at $BINARY${NC}"
    echo "Building in release mode..."
    cargo build --release
    BINARY="./target/release/find_ext"
fi

# Function to run benchmark
run_benchmark() {
    local use_cache=$1
    local label=$2

    export FIND_EXT_USE_CACHE=$use_cache

    # Clear cache before benchmark
    $BINARY --clear 2>/dev/null || true

    echo -e "${GREEN}Running: $label${NC}"

    local total_time=0
    local times=()

    for i in $(seq 1 $ITERATIONS); do
        start=$(date +%s%N)
        $BINARY "$BENCHMARK_DIR" > /dev/null
        end=$(date +%s%N)

        elapsed=$((($end - $start) / 1000000)) # Convert to milliseconds
        times+=($elapsed)
        total_time=$(($total_time + $elapsed))

        printf "  Run %2d: %4d ms\n" $i $elapsed
    done

    # Calculate average
    avg_time=$(($total_time / $ITERATIONS))

    # Calculate min and max
    min_time=${times[0]}
    max_time=${times[0]}
    for time in "${times[@]}"; do
        [ $time -lt $min_time ] && min_time=$time
        [ $time -gt $max_time ] && max_time=$time
    done

    echo ""
    echo -e "${YELLOW}  Average: $avg_time ms${NC}"
    echo "  Min: $min_time ms"
    echo "  Max: $max_time ms"
    echo ""

    # Clean up cache
    $BINARY --clear 2>/dev/null || true
}

# Run benchmarks
echo -e "${BLUE}--- Without Cache ---${NC}"
run_benchmark "false" "No Cache"

echo -e "${BLUE}--- With Cache (first run) ---${NC}"
run_benchmark "true" "With Cache"

echo -e "${BLUE}--- With Cache (warm cache) ---${NC}"
export FIND_EXT_USE_CACHE=true
# Warm up the cache
$BINARY "$BENCHMARK_DIR" > /dev/null

echo -e "${GREEN}Running: Warm Cache${NC}"
total_time=0
times=()

for i in $(seq 1 $ITERATIONS); do
    start=$(date +%s%N)
    $BINARY "$BENCHMARK_DIR" > /dev/null
    end=$(date +%s%N)

    elapsed=$((($end - $start) / 1000000))
    times+=($elapsed)
    total_time=$(($total_time + $elapsed))

    printf "  Run %2d: %4d ms\n" $i $elapsed
done

avg_time=$(($total_time / $ITERATIONS))
min_time=${times[0]}
max_time=${times[0]}
for time in "${times[@]}"; do
    [ $time -lt $min_time ] && min_time=$time
    [ $time -gt $max_time ] && max_time=$time
done

echo ""
echo -e "${YELLOW}  Average: $avg_time ms${NC}"
echo "  Min: $min_time ms"
echo "  Max: $max_time ms"
echo ""

# Clean up
$BINARY --clear 2>/dev/null || true

echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}Benchmark Complete!${NC}"
echo -e "${BLUE}========================================${NC}"
