#!/bin/zsh
# Everything a tag will be held to, checked here before it is pushed: the
# version in Cargo.toml and Cargo.lock, the changelog section, the notary
# credential, a clean tree, no tag of that name yet, the tests, and the
# build. A tag that fails is a tag redone, with the images built again.
#
#   scripts/release-check.sh 0.2.0
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
v="${1:?version, e.g. 0.2.0}"
fail() { print -u2 "release-check: $1"; exit 1; }

grep -q "^version = \"$v\"$" Cargo.toml || fail "Cargo.toml does not say version = \"$v\""
scripts/release-notes.sh "$v" > /dev/null || fail "CHANGELOG.md has no section for v$v"
# The notary service is asked now, with what release-mac.sh will use: a
# credential it cannot reach is found here, not after the tag is out.
said=$(xcrun notarytool history --keychain-profile "${TABFLIP_NOTARY_PROFILE:-notarytool-profile}" 2>&1) \
  || fail "the notary service cannot be reached with the saved credential (WORKFLOW.md, When something fails):
${said##*Error: }"
# --locked is how CI builds: a lock that was not refreshed fails here.
cargo check --locked -p tabflip-app -p tabflip-core 2>&1 | tail -1
[[ -z "$(git status --porcelain)" ]] || fail "uncommitted changes; the tag would not hold them"
git rev-parse -q --verify "refs/tags/v$v" > /dev/null && fail "the tag v$v exists already (WORKFLOW.md, redoing a release)"

cargo test --locked -p tabflip-core -p tabflip-app 2>&1 | grep -E "^test result|FAILED|panicked" || true
cargo test --locked -p tabflip-core -p tabflip-app > /dev/null 2>&1 || fail "the tests fail"
cargo build --locked -p tabflip-app 2>&1 | tail -1
print "release-check: v$v is ready to tag at $(git rev-parse --short HEAD)"
