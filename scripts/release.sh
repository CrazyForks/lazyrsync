#!/usr/bin/env bash
#
# Bump the AUR package to an already-released version.
#
# Run this AFTER `git push origin vX.Y.Z` has built the GitHub release.
# Homebrew is on homebrew-core and BrewTestBot bumps it from the tag.
# Publishing to crates.io stays a separate `cargo publish`.
#
#   scripts/release.sh 0.1.2
#
# Override the local clone location with AUR_DIR if needed.

set -euo pipefail

VERSION="${1:?usage: release.sh <version>   e.g. release.sh 0.1.2}"
REPO="westpoint-io/lazyrsync"
TAG="v$VERSION"
AUR_DIR="${AUR_DIR:-$HOME/aur-lazyrsync}"

SRC="https://github.com/$REPO/archive/refs/tags/$TAG.tar.gz"

sha() { curl -fsSL "$1" | sha256sum | cut -d' ' -f1; }

echo "==> verifying release $TAG exists"
gh release view "$TAG" --repo "$REPO" >/dev/null

[ -d "$AUR_DIR/.git" ] || git clone "ssh://aur@aur.archlinux.org/lazyrsync.git" "$AUR_DIR"

echo "==> AUR: bump to $VERSION (source build)"
src_sha="$(sha "$SRC")"
cd "$AUR_DIR"
git pull --quiet --ff-only origin master 2>/dev/null || true
sed -i \
  -e "s/^pkgver=.*/pkgver=$VERSION/" \
  -e "s/^pkgrel=.*/pkgrel=1/" \
  -e "s/^sha256sums=('.*')/sha256sums=('$src_sha')/" \
  PKGBUILD
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git -c commit.gpgsign=false commit -m "lazyrsync $VERSION"
git push origin HEAD:master

echo
echo "==> done. AUR is on $VERSION."
echo "    still to do:  cargo publish   (from the lazyrsync repo, for crates.io)"
