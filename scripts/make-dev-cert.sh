#!/bin/sh
# A signing certificate of your own for dev builds, "tabflip dev", made once
# and kept in the login keychain. Signed with it, every dev build is the
# same app to macOS, so Accessibility, once allowed, stays allowed.
# Not for releases: those are signed with a Developer ID (WORKFLOW.md).
#
#   scripts/make-dev-cert.sh
#
# macOS asks once to let codesign use the key: choose "Always Allow".
set -eu
name="tabflip dev"
if security find-identity -p codesigning 2>/dev/null | grep -q "\"$name\""; then
  echo "make-dev-cert: \"$name\" is already in the keychain"
  exit 0
fi
dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT
cat > "$dir/cert.cnf" <<CNF
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $name
[ext]
basicConstraints = critical, CA:false
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
CNF
/usr/bin/openssl req -x509 -newkey rsa:2048 -nodes -days 3650 -config "$dir/cert.cnf" -keyout "$dir/key.pem" -out "$dir/cert.pem" 2>/dev/null
/usr/bin/openssl pkcs12 -export -inkey "$dir/key.pem" -in "$dir/cert.pem" -name "$name" -passout pass:tabflip -out "$dir/dev.p12"
security import "$dir/dev.p12" -k "$HOME/Library/Keychains/login.keychain-db" -P tabflip -T /usr/bin/codesign >/dev/null
echo "make-dev-cert: \"$name\" added to the login keychain; scripts/dev-app.sh signs with it from now on"
