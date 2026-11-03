# Release

How a release of Tabflip is cut, command by command: the version, the changelog, the checks before the tag, the tag, what CI builds, signing and notarizing the Mac images, publishing, installing, cleaning up, and redoing a release. Tabflip is a Mac's alone, so a release is two disk images and nothing else. Each guard below is there because a release went wrong without it; "When something fails" says which.

## What a release is

A `v*` tag pushed to `alkaline-software/tabflip` runs `release.yml`. Two package jobs build the app on macOS (both architectures, from the Apple silicon runner), then one `release` job creates the GitHub Release as a **draft**, with the tag's section of `CHANGELOG.md` as its notes, and attaches the disk images under stable names:

| asset | built by | signed |
|---|---|---|
| `tabflip-mac-arm64.dmg` | CI, then replaced from the signer's Mac | Developer ID, notarized, stapled |
| `tabflip-mac-x64.dmg` | CI, then replaced from the signer's Mac | Developer ID, notarized, stapled |

The names never change, so `releases/latest/download/<name>` stays valid, and the app fetches its own update from it. CI's images are ad-hoc signed because the signing secrets are not in the repository; the notarized ones are built on the signer's Mac by `scripts/release-mac.sh` and uploaded over them. A release is not done until step 6 has published the draft: `releases/latest`, which the app's update check follows, names only published releases.

**Who does what.** Anyone with write access does steps 1 to 4. Steps 5 and 6 need the Developer ID certificate and a notarytool profile, which the signer has (at present Pingfan). Nothing personal is in the repository: the scripts find the certificate in the keychain, or take `TABFLIP_SIGN_IDENTITY` and `TABFLIP_NOTARY_PROFILE`.

## One-time setup on a Mac

Everyone:

- Rust via rustup (`curl https://sh.rustup.rs | sh`).
- Xcode's command line tools (`xcode-select --install`).
- `gh auth login` with an account that has write access to the repository.

The signer, in addition:

- `rustup target add x86_64-apple-darwin`
- `cargo install cargo-packager --locked`
- The certificate `Developer ID Application: Pingfan Hu (XC2WL5WN7J)` in the login keychain (`security find-identity -v -p codesigning` lists it).
- The notarytool keychain profile `notarytool-profile` (`xcrun notarytool store-credentials notarytool-profile` once, with the Apple ID, an app-specific password and team `XC2WL5WN7J`).

For the disk image's background only: `uv` (which fetches Pillow itself) and the LXGW WenKai Medium font in `~/Library/Fonts`. The icon needs nothing beyond a Mac (`swift` ships with it).

A developer without a Developer ID can still keep Accessibility granted across dev builds: `scripts/make-dev-cert.sh` once (`docs/platform.md`, Dev builds and signing).

## Each release

### 1. Bump the version

The version lives in one file, the workspace's `Cargo.toml` (`version = "X.Y.Z"`); both crates inherit it. `Cargo.lock` records the crates' versions too, and CI builds with `--locked`, so refresh it after the bump and commit it with the rest:

```
cargo check -p tabflip-app
```

The release workflow refuses a tag whose number disagrees with `Cargo.toml`.

### 2. Write the changelog section

Add `## vX.Y.Z` to `CHANGELOG.md`, newest first, written for the person installing the app: what changed for them, not which files moved. A numbered list, a topic in bold and then one sentence, small fixes gathered under **Fixes**, ten items at most. The workflow refuses a tag with no section, and `scripts/release-notes.sh X.Y.Z` prints what the release page will show.

### 3. Keep the display on, and check it before the tag

Commit the bump and the changelog, keep the Mac's display on until step 5 is done, then:

```
scripts/release-check.sh X.Y.Z
```

It holds the commit to everything the tag will be held to: the version in `Cargo.toml`, a changelog section, a notary service that answers to the saved credential (step 5 needs it, half an hour later), a lock that builds with `--locked`, a clean tree, no tag of that name yet, the core's tests, the app's tests (the languages' table), and the app build. Do not tag until it says "ready to tag". A tag that fails in CI has to be redone, and every redo means building and notarizing the Mac images again.

### 4. Push, wait for the tests, then tag

`rust.yml` runs on a push to `main`: the core's tests and an app build on a clean runner, which is the one place a lock or a file left out of the commit is caught. Push, let that run finish, and tag only once it is green:

```
git push origin main
gh run list --repo alkaline-software/tabflip --workflow rust --limit 1
gh run watch <run-id> --repo alkaline-software/tabflip --exit-status
git tag -a vX.Y.Z -m "Tabflip vX.Y.Z"
git push origin refs/tags/vX.Y.Z
```

The Mac images (step 5) can be built while it runs: the commit is the one that will be tagged, unless the tests send you back.

The tag push starts `release.yml`. It takes about twenty minutes (the cold GPUI build) and ends with a **draft** release holding CI's images: nothing is public, and `releases/latest` still names the previous version. Watch it:

```
gh run list --repo alkaline-software/tabflip --workflow release --limit 3
gh run watch <run-id> --repo alkaline-software/tabflip --exit-status
```

### 5. Build, sign and notarize the Mac images on the signer's Mac

While CI runs, and from the tagged commit: an image built before the last fix is an image of other code, and `release-publish.sh` refuses one older than the tag's commit. Both architectures build on an Apple silicon Mac. The script packages the bundle with cargo-packager, signs it with the Developer ID and the hardened runtime, lays the disk image out through Finder over `scripts/dmg/background.png`, compresses it, signs it, submits it to the notary service, waits, staples and validates:

```
cargo build --release --locked -p tabflip-app
cargo build --release --locked -p tabflip-app --target x86_64-apple-darwin
scripts/release-mac.sh --no-build --notarize
scripts/release-mac.sh --no-build --notarize --target x86_64-apple-darwin
```

Each run ends with `built dist/tabflip-mac-<arch>.dmg (X.Y.Z, <arch>, signed, notarized)` and leaves only the two images in `dist/`. Notarization takes a few minutes an image; the four commands together about ten. Check each run's exit code and last line.

### 6. Put them on the release and publish it

Once the workflow is green and both images are built:

```
scripts/release-publish.sh X.Y.Z
```

It refuses unless the release workflow for the tag has finished and succeeded (`action-gh-release` replaces same-named assets, so an upload made while the job runs is overwritten by it) and both images are notarized, stapled and newer than the tag's commit. Then it uploads them over CI's ad-hoc ones, downloads them back and compares checksums, checks the downloaded copies the way Gatekeeper will, and publishes the draft as the latest release. The release is not done until this prints "published with notarized Mac images".

### 7. Install the release build

Drag from the disk image, or:

```
hdiutil attach dist/tabflip-mac-arm64.dmg
ditto "/Volumes/Tabflip vX.Y.Z/Tabflip.app" /Applications/Tabflip.app
hdiutil detach "/Volumes/Tabflip vX.Y.Z"
open /Applications/Tabflip.app
```

Tabflip opens with no window and no Dock icon, only its item in the menu bar, and takes ⌘Tab as soon as Accessibility is given. The release build has its own entry in Privacy & Security ("Tabflip"), apart from the dev build's ("Tabflip dev"), so its first launch opens Settings on General, where Allow is. A later release, signed with the same Developer ID, keeps the grant. Two copies never run at once (`docs/platform.md`, One Tabflip at a time): `scripts/dev-app.sh` quits an installed copy before opening the build, and step 8 hands back.

### 8. Delete the compiled files

Every release ends with this step, without being asked, once the release is confirmed done: step 6 printed "published with notarized Mac images" and the installed copy opens. Nothing compiled is kept between releases; disk space is worth more than the minutes the next build takes.

```
scripts/release-clean.sh
```

- It runs `cargo clean`, which deletes `target/`: every build made while developing and for the release (debug, release, the Intel target), and with it the dev build's bundle.
- It deletes `dist/`, the two disk images, which are on the release by now.
- It asks Spotlight for every `Tabflip.app` and fails if there is one other than `/Applications/Tabflip.app`.
- If the dev build was running it quits it, since it has no files left, and opens `/Applications/Tabflip.app` in its place.

It refuses in two cases: the installed copy is not a notarized release, or `dist/` holds images of a version that is not published (a redo of the release wants them).

## Redoing a release under the same tag

Delete the release and the tag on both sides, cancel a run still going (`gh run cancel <run-id>`), then repeat from step 3 with the new commit:

```
gh release delete vX.Y.Z --yes --repo alkaline-software/tabflip
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
git tag -a vX.Y.Z -m "Tabflip vX.Y.Z"
git push origin refs/tags/vX.Y.Z
```

The Mac images are built again from the new commit (step 5) and published again (step 6). A draft that will not be published is deleted (`gh release delete vX.Y.Z --yes`): the public never sees a draft, but it stands over the published releases in the maintainers' view.

## What stays fixed

- **The bundle identifier** `com.alkalinesoftware.tabflip` (`crates/tabflip-app/Cargo.toml`): a change makes macOS treat Tabflip as a new app, and everyone must allow Accessibility again.
- **The names machines know, in lower case**: the repository, the release's asset names (`tabflip-mac-arm64.dmg`, which every copy out there fetches its update by), the settings folder, `TABFLIP_*`, the crates and the binary. The app's name is Tabflip, with a capital, everywhere a person reads it: the bundle (`Tabflip.app`), its menus, the disk image. It was tabflip until v0.2.4. A copy from before looks for `tabflip.app` in the image it updates from, so the image is a case-insensitive HFS+ volume (`release-mac.sh`), and that copy keeps its lowercase folder until its next update, which takes the new name (`update::placed_as`).

## The pieces a release carries

- **The icon.** `scripts/icon/icon.html` is the drawing and `scripts/make-icon.sh` renders it into `crates/tabflip-app/assets/` (`docs/design.md`, The app icon). Change the drawing, run the script, rebuild, commit the assets.
- **The disk image's background.** `scripts/dmg/background.png`, drawn by `scripts/dmg/background.py` (Pillow, LXGW WenKai Medium) on the window's page color with the sapphire: "Drag Tabflip to Applications" and an arrow, 1320 by 840 at 2x for the 660 by 420 Finder window. Run the script only when the picture changes (`uv run --with pillow python3 scripts/dmg/background.py`); the PNG is committed.
- **The layout.** `release-mac.sh` mounts a read-write image and has Finder set the window, the 128 pt icons at 165 and 495, and the background, then converts it to a compressed read-only image. Only Finder writes a `.DS_Store` Finder honors, so the layout is not generated offline. This works on the GitHub macOS runners too.
- **The update inside the app.** About in Settings fetches the newer version's image from the release's stable asset name (`update::download`), mounts it, copies `Tabflip.app` beside the running bundle and swaps it in (`update::install`), then quits; the new app opens once the old process has ended. `tabflip --try-update <bundle>` does the same to a throwaway copy of the bundle and opens nothing, to try the steps short of the relaunch.

## When something fails

Every failure met while cutting a release gets a line here once it is solved, with what would have caught it, and the steps above are changed so it is caught before the tag next time.

- **A tag pushed with the tests still running** went red and was redone, images and all: step 4 waits for `rust.yml` before tagging.
- **One Mac package job fails right after "satisfies its Designated Requirement", with no message, while the other passes**: `hdiutil create` on the runner ("Resource busy"). `release-mac.sh` tries it five times and says why. If it still fails, re-run only that job, which keeps the tag and the images built on the signer's Mac: `gh run rerun <run-id> --failed --repo alkaline-software/tabflip`.
- **`release-mac.sh` stops with "Finder ... AppleEvent timed out (-1712)"**: macOS asked whether the terminal may control Finder and nobody answered. Allow it (System Settings, Privacy & Security, Automation) and run the script again. Check the script's exit code and that both images are in `dist/`.
- **"No Keychain password item found for profile: notarytool-profile"** while the profile is in place: the Mac's screen is locked. The notary profile is in a keychain that opens only while the screen is unlocked, and a Mac that locks when the display sleeps loses it, in an agent's session or anyone's, while builds and CI go on. Keep the display on (step 3) and notarize while the person is at the Mac; `release-check.sh` asks the notary service before anything is built.
- **`git push origin vX.Y.Z` stops with "src refspec vX.Y.Z matches more than one"**: a branch has the tag's name. Push the tag by its full name, `git push origin refs/tags/vX.Y.Z`, as step 4 does, and name a release's branch apart from its tag.
- **"Cargo.toml says X but the tag is vY"**: the version and the tag disagree. Fix `Cargo.toml`, commit, redo the tag.
- **"CHANGELOG.md has no section '## vX.Y.Z'"**: add it, commit, redo the tag.
- **`--locked` fails in CI after a bump**: `Cargo.lock` was not refreshed. `cargo check -p tabflip-app`, commit the lock.
- **Notarization rejected**: `xcrun notarytool log <submission-id> --keychain-profile notarytool-profile`. The usual causes are a binary signed without `--options runtime` or an unsigned helper inside the bundle; the script signs with `--deep` and the hardened runtime, so look for a new binary the packager added.
- **Two Tabflips in Launchpad or Spotlight**: a second bundle on the disk. `mdfind "kMDItemFSName == 'tabflip.app'c"` (the `c` matches without case: an older copy is `tabflip.app`) lists them; keep only `/Applications/Tabflip.app`. The dev build's bundle is made in `target/debug/dev.noindex/`, which Spotlight passes by, and step 8 removes it with the rest of `target/`.
- **The Mac images on the release are ad-hoc signed**, or macOS says the app "cannot be opened" after an update: step 6 was skipped. Run `scripts/release-publish.sh X.Y.Z`. To check an installed copy: `spctl -a -vv /Applications/Tabflip.app` must say "Notarized Developer ID"; "rejected" with `Signature=adhoc` from `codesign -dv` is CI's image. Reinstall from `dist/` (step 7).
- **The app's update check finds nothing after a release**: the release is still a draft. `releases/latest` names only published releases; run step 6.
- **Tags published as drafts and never published** (v0.1.0 to v0.1.3, before there was a Developer ID): their drafts held only CI's ad-hoc images and were deleted; the tags and the changelog sections stay.
- **"Node.js NN is deprecated" on a run**: an action is a major version behind. `gh api repos/<owner>/<action>/releases/latest -q .tag_name` gives the current one; move `rust.yml` and `release.yml` together.
