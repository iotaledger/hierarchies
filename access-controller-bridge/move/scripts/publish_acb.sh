#!/bin/bash

# Copyright 2026 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0

# Publishes the Access Controller Bridge Move package.
# Expects the Hierarchies and TfComponents packages to be published first,
# with their addresses resolved via Move.toml dependency paths.

set -e

script_dir=$(cd "$(dirname "$0")" && pwd)
package_dir="$script_dir/.."

echo "Publishing Access Controller Bridge from $package_dir"

RESPONSE=$(iota client publish --with-unpublished-dependencies --silence-warnings --json --gas-budget 500000000 "$package_dir")
{ # try
    PACKAGE_ID=$(echo "$RESPONSE" | jq --raw-output '.objectChanges[] | select(.type | contains("published")) | .packageId')
} || { # catch
    echo "$RESPONSE"
    exit 1
}

if [ -z "$PACKAGE_ID" ]; then
    echo "Failed to extract package ID from response"
    echo "$RESPONSE"
    exit 1
fi

export IOTA_ACB_PKG_ID=$PACKAGE_ID
echo "IOTA_ACB_PKG_ID=${IOTA_ACB_PKG_ID}"
