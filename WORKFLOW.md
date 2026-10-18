# Releasing tabflip

Everything about cutting a release: bumping the version, the changelog,
the checks before the tag, the tag, what CI builds, signing and
notarizing the Mac images on this machine, uploading them, installing,
cleaning up, redoing a release, and the icon the bundle carries. It is
Kotoba's release with the names changed and the pieces tabflip has no
use for left out (tabflip is a Mac's alone: no Windows or Linux
installer, no MinGW check). AGENTS.md says how the pieces fit; this file
says what to run, in order. The lessons under "When something fails" are
Emaki's and Kotoba's: each guard below is there because a release went
wrong without it.

## What a release is

A `v*` tag pushed to `alkaline-software/tabflip` runs `release.yml`. Two
package jobs build the app on macOS (both architectures, from the Apple
Silicon runner), then one `release` job creates the GitHub Release, as a
draft, with the tag's section of `CHANGELOG.md` as its notes and attaches
the disk images under stable names:

| asset | built by | signed |
|---|---|---|
| `tabflip-mac-arm64.dmg` | CI, then replaced from this machine | Developer ID, notarized, stapled |
| `tabflip-mac-x64.dmg` | CI, then replaced from this machine | Developer ID, notarized, stapled |

The names never change, so `releases/latest/download/<name>` links stay
valid. CI's own images are ad-hoc signed because the signing secrets are
not in the repository; the notarized ones are built here by
`scripts/release-mac.sh` and uploaded over them. A release is not done
until step 6 has published the draft: `releases/latest`, which the app's
own update check follows, names only published releases.

**Who does what.** Anyone with write access does steps 1 to 4. Steps 5
and 6 need the Developer ID certificate and a notarytool profile, which
Pingfan has. Nothing personal is in the repository: the scripts find the
certificate in the keychain, or take `TABFLIP_SIGN_IDENTITY` and
`TABFLIP_NOTARY_PROFILE`.

## One-time setup on a Mac

Everyone:

- Rust via rustup (`curl https://sh.rustup.rs | sh`).
- Xcode's command line tools (`xcode-select --install`).
- `gh auth login` with an account that has write access to the repository.

The signer, in addition:

- `rustup target add x86_64-apple-darwin`
- `cargo install cargo-packager --locked`
- The certificate `Developer ID Application: Pingfan Hu (XC2WL5WN7J)` in
  the login keychain (`security find-identity -v -p codesigning` lists it).
- The notarytool keychain profile `notarytool-profile`
  (`xcrun notarytool store-credentials notarytool-profile` once, with the
  Apple ID, an app-specific password and team `XC2WL5WN7J`).

For the disk-image background only: `uv` (which fetches Pillow itself)
and the LXGW WenKai Medium font in `~/Library/Fonts`. The icon needs
nothing beyond a Mac (`swift` ships with it).

A developer without a Developer ID can still keep Accessibility granted
across dev builds: `scripts/make-dev-cert.sh` once (AGENTS.md).

## Each release

### 1. Bump the version

The version lives in one file:

```
Cargo.toml            version = "X.Y.Z"      (the workspace; both crates inherit it)
```

`Cargo.lock` records the crates' versions too, and CI builds with
`--locked`, so refresh it after the bump and commit it with the rest:

```
cargo check -p tabflip-app
```

The release workflow refuses a tag whose number disagrees with `Cargo.toml`.

### 2. Write the changelog section

Add `## vX.Y.Z` to `CHANGELOG.md`, newest first, written for the person
installing the app: what changed for them, not which files moved, one
line per change, with a short "For developers" part after it where a
change matters to someone building the app. The workflow refuses a tag
with no section, and `scripts/release-notes.sh X.Y.Z` prints exactly
what the release page will show.

### 3. Check it before the tag

Commit the bump and the changelog, then:

```
scripts/release-check.sh X.Y.Z
```

It holds the commit to everything the tag will be held to: the version in
`Cargo.toml`, a changelog section, a notary service that answers to the
saved credential (step 5 needs it, half an hour later), a lock that builds
with `--locked`, a clean tree, no tag of that name yet, the core tests,
the app's tests (the languages' table), and the app build. Do not tag
until it says "ready to tag". A tag that fails in CI has to be redone,
and every redo means building and notarizing the Mac images again.

### 4. Push, wait for the tests, then tag

`rust.yml` runs on a push to `main`: the core tests and an app build on a
clean runner, which is the one place a lock or a file left out of the
commit is caught. So push, let that run finish, and tag only once it is
green:

```
git push origin main
gh run list --repo alkaline-software/tabflip --workflow rust --limit 1
gh run watch <run-id> --repo alkaline-software/tabflip --exit-status
git tag -a vX.Y.Z -m "tabflip vX.Y.Z"
git push origin refs/tags/vX.Y.Z
```

The Mac images (step 5) can be built while it runs: the commit is the
one that will be tagged, unless the tests send you back.

The tag push starts `release.yml`. It takes about twenty minutes (the
cold GPUI build), and ends with a **draft** release holding CI's images:
nothing is public, and `releases/latest` still names the previous
version. Watch it:

```
gh run list --repo alkaline-software/tabflip --workflow release --limit 3
gh run watch <run-id> --repo alkaline-software/tabflip --exit-status
```

### 5. Build, sign and notarize the Mac images here

While CI runs, and from the tagged commit: an image built before the last
fix is an image of other code, and `release-publish.sh` refuses one older
than the tag's commit. Both architectures build on an Apple Silicon Mac.
The script packages the bundle with cargo-packager, signs it with the
Developer ID and the hardened runtime, lays the disk image out through
Finder over `scripts/dmg/background.png`, compresses it, signs it,
submits it to the notary service, waits, staples and validates:

```
cargo build --release --locked -p tabflip-app
cargo build --release --locked -p tabflip-app --target x86_64-apple-darwin
scripts/release-mac.sh --no-build --notarize
scripts/release-mac.sh --no-build --notarize --target x86_64-apple-darwin
```

Each run ends with `built dist/tabflip-mac-<arch>.dmg (X.Y.Z, <arch>,
signed, notarized)` and leaves only the two images in `dist/`.
Notarization takes a few minutes per image; the four commands together
about ten.

Check each run's own exit code and last line. Step 3 asked the notary
service already, so a credential that cannot be reached here is new since
then: see "When something fails".

### 6. Put them on the release and publish it

Once the workflow is green and both images are built:

```
scripts/release-publish.sh X.Y.Z
```

It refuses unless the release workflow for the tag has finished and
succeeded (`action-gh-release` replaces same-named assets, so an upload
made while the job runs is overwritten by it) and both images are
notarized, stapled and newer than the tag's commit. Then it uploads them
over CI's ad-hoc ones, downloads them back and compares checksums, checks
the downloaded copies the way Gatekeeper will, and publishes the draft as
the latest release. The release is not done until this prints "published
with notarized Mac images".

### 7. Install the release build here

Drag from the disk image, or:

```
hdiutil attach dist/tabflip-mac-arm64.dmg
ditto "/Volumes/tabflip vX.Y.Z/tabflip.app" /Applications/tabflip.app
hdiutil detach "/Volumes/tabflip vX.Y.Z"
open /Applications/tabflip.app
```

tabflip is a menu bar app: it opens with no window and no Dock icon,
only its item in the menu bar, and takes ⌘Tab as soon as Accessibility
is given. The release build has its own entry in Privacy & Security
("tabflip"), apart from the dev build's ("tabflip dev"), so its first
launch opens Settings on General, where Allow is. A later release,
signed with the same Developer ID, keeps the grant. Two copies must not
run at once, since both would take the shortcut: `scripts/dev-app.sh`
quits an installed copy before opening the build, and step 8 hands back.

Keep no other `tabflip.app` on the disk (`dist/`, a scratch folder):
Launchpad and Spotlight list each one. Step 8 removes the ones a release
leaves.

### 8. Delete the compiled files

Every release ends with this step, and a release is not finished until it
has run. Nothing compiled is kept between releases: disk space is worth
more here than the minutes the next build takes. Do it without being
asked, once the release is confirmed done: step 6 printed "published with
notarized Mac images" and the installed copy opens. Then:

```
scripts/release-clean.sh
```

- It runs `cargo clean`, which deletes `target/`: every build made while
  developing and for the release (debug, release, the Intel target), and
  with it the dev build's bundle.
- It deletes `dist/`, the two disk images, which are on the release by
  now.
- It asks Spotlight for every `tabflip.app` and fails if there is one
  other than `/Applications/tabflip.app`.
- If the dev build was running it quits it, since it has no files left,
  and opens `/Applications/tabflip.app` in its place.

It refuses in two cases: the installed copy is not a notarized release,
or `dist/` holds images of a version that is not published (a redo of
the release wants them and the release builds).

Nothing here can be lost: all of it is rebuilt from the source. The cost
is time. The next `cargo build -p tabflip-app` starts from nothing and
takes some minutes, and so does the next release build.

## Redoing a release under the same tag

Delete the release and the tag on both sides, cancel a run still going
(`gh run cancel <run-id>`), then repeat from step 3 with the new commit:

```
gh release delete vX.Y.Z --yes --repo alkaline-software/tabflip
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
git tag -a vX.Y.Z -m "tabflip vX.Y.Z"
git push origin refs/tags/vX.Y.Z
```

The Mac images must be rebuilt from the new commit (step 5) and published
again (step 6).

## Before the first public release

- The bundle identifier `com.alkalinesoftware.tabflip`
  (`crates/tabflip-app/Cargo.toml`) is fixed from the first published
  release on. A change after it makes macOS treat tabflip as a new app,
  and everyone must allow Accessibility again.
- v0.1.0 to v0.1.3 were tagged and built but never published (step 6
  was not run, the signer having no Developer ID then), and their drafts
  held only CI's ad-hoc images, so they were deleted; the tags and the
  changelog sections stay. A draft that will not be published is deleted
  (`gh release delete vX.Y.Z --yes`): the public never sees a draft, but
  it stands over the published releases in the maintainers' view.

## The pieces a release depends on

**The icon.** `scripts/icon/icon.html` is the drawing, on a canvas, with
proofs of every size and the candidates it was chosen from;
`scripts/make-icon.sh` renders it through WebKit
(`scripts/icon/render.swift`) into `crates/tabflip-app/assets/icon/` (the
PNGs as the rounded icon with its margin and shadow, and `icon.icns` as
the full square, opaque to its corners, because macOS 26 masks each icon
to its own rounded shape and shows anything transparent as a grey border)
and the menu bar's template image into `assets/icons/` (`menubar.png` at
36 px, `menubar-1x.png` at 18). Change the drawing, run the script,
rebuild, commit the assets. `docs/design.md`, The app icon.

**The disk-image background.** `scripts/dmg/background.png`, drawn by
`scripts/dmg/background.py` (Pillow, LXGW WenKai Medium) on the window's
page colour with the lavender: "Drag tabflip to Applications" and an
arrow, 1320x840 at 2x for the 660x420 Finder window. Run the script only
when the picture changes (`uv run --with pillow python3
scripts/dmg/background.py`); the PNG is committed.

**The layout.** `release-mac.sh` mounts a read-write image and has Finder
set the window, the 128pt icons at 165 and 495, and the background, then
converts to a compressed read-only image. Only Finder writes a `.DS_Store`
Finder honours, so the layout is not generated offline. This works on the
GitHub macOS runners too.

**The update inside the app.** Updates in Settings fetches the newer
version's image from the release's stable asset name
(`update::download`), mounts it, copies `tabflip.app` beside the running
bundle and swaps it in (`update::install`), then quits; the new app opens
once the old process has ended. `tabflip --try-update <bundle>` does the
same to a throwaway copy of the bundle and opens nothing, to try the
steps short of the relaunch.

## When something fails

Every failure met while cutting a release gets a line here once it is
solved, with what would have caught it, and the steps above are changed
so it is caught before the tag next time.

- **A tag was pushed with the tests still running** and they went red:
  step 4 waits for `rust.yml` before tagging. A Kotoba release was
  tagged with its tests running and redone, images and all.
- **One Mac package job in `release.yml` fails right after "satisfies
  its Designated Requirement", with no message, while the other
  passes**: `hdiutil create` on the runner ("Resource busy").
  `release-mac.sh` tries it five times and says why. If it still fails,
  re-run only that job, which keeps the tag and the images built here:
  `gh run rerun <run-id> --failed --repo alkaline-software/tabflip`.
- **`release-mac.sh` stops with "Finder ... AppleEvent timed out
  (-1712)"**: macOS asked whether the terminal may control Finder and
  nobody answered. Allow it (System Settings, Privacy & Security,
  Automation) and run the script again. Check the script's exit code and
  that both images are in `dist/`.
- **`release-mac.sh --notarize` stops with "No Keychain password item
  found for profile: notarytool-profile"**, run by an agent, while the
  same two commands work in the person's own terminal: the credential is
  in place and could not be reached from the agent's session at that
  moment. Run the two commands of step 5 in a terminal of your own, then
  go on from step 6. `release-check.sh` asks the notary service before
  anything is built, so step 3 says so.
- **`git push origin vX.Y.Z` stops with "src refspec vX.Y.Z matches
  more than one"**: a branch has the tag's name. Push the tag by its full
  name, `git push origin refs/tags/vX.Y.Z`, as step 4 does, and name a
  release's branch apart from its tag.
- **"Cargo.toml says X but the tag is vY"**: the version and the tag
  disagree. Fix `Cargo.toml`, commit, redo the tag.
- **"CHANGELOG.md has no section '## vX.Y.Z'"**: add it, commit, redo
  the tag.
- **`--locked` fails in CI after a bump**: `Cargo.lock` was not
  refreshed. `cargo check -p tabflip-app`, commit the lock.
- **Notarization rejected**: `xcrun notarytool log <submission-id>
  --keychain-profile notarytool-profile`. The usual causes are a binary
  signed without `--options runtime` or an unsigned helper inside the
  bundle; the script signs with `--deep` and the hardened runtime, so
  look for a new binary the packager added.
- **Two tabflips in Launchpad or Spotlight**: a second bundle somewhere
  on disk. `mdfind "kMDItemFSName == 'tabflip.app'"` lists them; keep
  only `/Applications/tabflip.app`. The dev build's bundle is made in
  `target/debug/dev.noindex/`, which Spotlight passes by, and step 8
  removes it with the rest of `target/`.
- **The Mac images on the release are ad-hoc signed**, or macOS says the
  app "cannot be opened" after an update: step 6 was skipped. Run
  `scripts/release-publish.sh X.Y.Z`. To check an installed copy:
  `spctl -a -vv /Applications/tabflip.app` must say "Notarized Developer
  ID"; "rejected" with `Signature=adhoc` from `codesign -dv` is CI's
  image. Reinstall from `dist/` (step 7).
- **The app's update check finds nothing after a release**: the release
  is still a draft. `releases/latest` names only published releases;
  run step 6.
- **"Node.js NN is deprecated" on a run**: an action is a major version
  behind. `gh api repos/<owner>/<action>/releases/latest -q .tag_name`
  gives the current one; move `rust.yml` and `release.yml` together.
