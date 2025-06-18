#!/bin/bash

# Script to run all ITH examples
# Usage: ./run.sh
# Make sure to set IOTA_ITH_PKG_ID environment variable

if [ -z "$IOTA_ITH_PKG_ID" ]; then
    echo "Error: IOTA_ITH_PKG_ID environment variable is not set"
    echo "Usage: IOTA_ITH_PKG_ID=0x... ./run.sh"
    exit 1
fi

echo "Running all ITH examples..."
echo "Package ID: $IOTA_ITH_PKG_ID"
echo "================================"

examples=(
    "getting_started"
    "01_create_federation"
    "01_get_attestations_and_accreditations"
    "02_validate_statements"
    "03_get_statements"
    "02_add_root_authority"
    "03_add_and_remove_statement"
    "04_create_accreditation_to_attest"
    "05_revoke_accreditation_to_attest"
    "06_create_accreditation_to_accredit"
    "07_revoke_accreditation_to_accredit"
)

for example in "${examples[@]}"; do
    echo ""
    echo "Running: $example"
    echo "------------------------"
    cargo run --release --example "$example"
    if [ $? -ne 0 ]; then
        echo "Error: Failed to run $example"
        exit 1
    fi
done

echo ""
echo "All examples completed successfully!"
