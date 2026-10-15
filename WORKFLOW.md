# Releasing tabflip

A `v*` tag pushed to `alkaline-software/tabflip` runs `release.yml`. It
builds both Mac disk images ad-hoc signed and attaches them to a **draft**
release with the tag's section of `CHANGELOG.md` as its notes. The
Developer ID holder then builds the notarized images, puts them over CI's,
and publishes.

| asset | built by | signed |
|---|---|---|
| `tabflip-mac-arm64.dmg` | CI, then replaced by the signer | Developer ID, notarized, stapled |
| `tabflip-mac-x64.dmg` | CI, then replaced by the signer | Developer ID, notarized, stapled |

The names never change, so `releases/latest/download/<name>` links stay
valid.

**Who does what.** Anyone with write access does steps 1–3. Steps 4–5 need
the Developer ID certificate and a notarytool profile, which Pingfan has
(the same ones emaki ships with). Nothing personal is in the repository:
the scripts find the certificate in the keychain, or take
`TABFLIP_SIGN_IDENTITY` and `TABFLIP_NOTARY_PROFILE`.

## One-time setup

Everyone:

- Rust via rustup (`curl https://sh.rustup.rs | sh`).
- Xcode's command line tools (`xcode-select --install`).
- `gh auth login` with write access to the repository.

The signer, in addition:

- `rustup target add x86_64-apple-darwin`
- `cargo install cargo-packager --locked`
- The `Developer ID Application` certificate in the login keychain
  (`security find-identity -v -p codesigning` lists it).
- A notarytool keychain profile, `notarytool-profile` by default
  (`xcrun notarytool store-credentials notarytool-profile`).

A developer without a Developer ID can still keep Accessibility granted
across dev builds: `scripts/make-dev-cert.sh` once (AGENTS.md).

## Each release

1. **Bump the version** in the root `Cargo.toml` (`[workspace.package]`),
   then `cargo build` so `Cargo.lock` follows. CI builds with `--locked`.
2. **Write the notes**: a `## vX.Y.Z` section at the top of `CHANGELOG.md`.
   `scripts/release-notes.sh X.Y.Z` prints it; the tag's CI fails without
   one.
3. **Commit, tag, push**:
   ```
   git commit -am "vX.Y.Z" && git tag vX.Y.Z && git push && git push --tags
   ```
   Wait for the `release` workflow to go green. The release is a draft.
4. **Signer: build the notarized images** from the tag:
   ```
   git fetch --tags && git checkout vX.Y.Z
   scripts/release-mac.sh --notarize
   scripts/release-mac.sh --target x86_64-apple-darwin --notarize
   ```
   Each writes `dist/tabflip-mac-<arch>.dmg` and checks the staple.
5. **Signer: publish**:
   ```
   scripts/release-publish.sh X.Y.Z
   ```
   It refuses unless CI is green for the tag and both images are notarized
   and newer than the tag's commit, uploads them over CI's, checks the
   uploaded ones match, and publishes the draft.

## Before the first release

- Confirm the bundle identifier `com.alkalinesoftware.tabflip`
  (`crates/tabflip-app/Cargo.toml`) with the signer. A change after release
  makes macOS treat tabflip as a new app, and everyone must allow
  Accessibility again.

## The icon

`scripts/icon/draw.swift` draws it with AppKit at every size;
`scripts/make-icon.sh` writes `crates/tabflip-app/assets/icon/` (the PNGs
and `icon.icns`). The `.icns` is the full square, opaque to its corners:
macOS 26 masks each icon to its own rounded shape.
