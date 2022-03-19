#!/usr/bin/env bash

set -ex

fetch_version() {
    curl -fsSL "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/po/we/$1" \
        | tail -n1 | jq -r '.vers'
}

if [ "$CI" != true ]; then
    echo "Error: this script only works in GitHub CI"
    exit 1
fi

VERSION="${GITHUB_REF#refs/tags/}"

cargo publish --manifest-path crates/detach/Cargo.toml

while [ "$(fetch_version powerpack-detach)" != "$VERSION" ]; do
    sleep 30
done

cargo publish

cargo publish --manifest-path crates/cli/Cargo.toml
