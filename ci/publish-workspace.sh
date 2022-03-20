#!/usr/bin/env bash

set -ex

fetch_version() {
    curl -fsSL -H 'User-Agent: GitHub Actions (github.com/rossmacarthur/powerpack)' \
        "https://crates.io/api/v1/crates/$1" | jq -r '.versions[].num' | head -n1
}

if [ "$CI" != true ]; then
    echo "Error: this script only works in GitHub CI"
    exit 1
fi

VERSION="${GITHUB_REF#refs/tags/}"

cargo publish --manifest-path crates/detach/Cargo.toml

while [ "$(fetch_version powerpack-detach)" != "$VERSION" ]; do
    sleep 15
done

cargo publish

cargo publish --manifest-path crates/cli/Cargo.toml
