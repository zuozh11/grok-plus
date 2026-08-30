#!/bin/sh
set -eu

REPO="${GROK_PLUS_REPO:-zuozh11/grok-plus}"
HOME_DIR="${GROK_PLUS_HOME:-$HOME/.local/share/grok-plus}"
BIN_DIR="${GROK_PLUS_BIN_DIR:-$HOME/.local/bin}"
RELEASES_DIR="$HOME_DIR/releases"
CURRENT_LINK="$HOME_DIR/current"
COMMAND_PATH="$BIN_DIR/grok-plus"
UPDATER="$HOME_DIR/update"
ASSET="grok-plus-aarch64-apple-darwin.tar.gz"

force=0
check=0
quiet=0
requested_version=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --force) force=1 ;;
    --check) check=1 ;;
    --quiet) quiet=1 ;;
    --version)
      shift
      requested_version="${1:?--version requires a value}"
      ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
  shift
done

log() {
  [ "$quiet" -eq 1 ] || printf '%s\n' "$*"
}

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  echo "grok-plus currently supports Apple Silicon macOS only." >&2
  exit 1
fi

current_build=""
if [ -f "$CURRENT_LINK/BUILD_ID" ]; then
  current_build="$(cat "$CURRENT_LINK/BUILD_ID")"
elif [ -f "$CURRENT_LINK/VERSION" ]; then
  current_build="$(cat "$CURRENT_LINK/VERSION")"
fi

if [ -n "$requested_version" ]; then
  tag="$requested_version"
  base_url="https://github.com/$REPO/releases/download/$tag"
else
  latest_url="$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest")"
  tag="${latest_url##*/}"
  base_url="https://github.com/$REPO/releases/latest/download"
fi
build_id="$tag"

if [ "$check" -eq 1 ]; then
  if [ "$current_build" = "$build_id" ]; then
    log "grok-plus is up to date ($build_id)."
    exit 0
  fi
  log "grok-plus update available: ${current_build:-not installed} -> $build_id"
  exit 0
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/grok-plus.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

mkdir -p "$RELEASES_DIR" "$BIN_DIR" "$HOME_DIR"
release_dir="$RELEASES_DIR/$build_id"
if [ "$force" -eq 1 ] || [ ! -x "$release_dir/grok-plus" ]; then
  archive="$tmp_dir/$ASSET"
  checksum="$tmp_dir/$ASSET.sha256"
  curl -fsSL "$base_url/$ASSET" -o "$archive"
  curl -fsSL "$base_url/$ASSET.sha256" -o "$checksum"
  (cd "$tmp_dir" && shasum -a 256 -c "$ASSET.sha256" >/dev/null)
  tar -xzf "$archive" -C "$tmp_dir"
  if [ "$(cat "$tmp_dir/BUILD_ID")" != "$build_id" ]; then
    echo "Release tag and archive build ID do not match." >&2
    exit 1
  fi
  stage="$RELEASES_DIR/.${build_id}.tmp.$$"
  rm -rf "$stage"
  mkdir -p "$stage"
  cp "$tmp_dir/grok-plus" "$stage/grok-plus"
  cp "$tmp_dir/BUILD_ID" "$stage/BUILD_ID"
  chmod 755 "$stage/grok-plus"
  rm -rf "$release_dir"
  mv "$stage" "$release_dir"
fi

rm -f "$HOME_DIR/.current.new"
ln -s "$release_dir" "$HOME_DIR/.current.new"
/bin/mv -fh "$HOME_DIR/.current.new" "$CURRENT_LINK"

cat > "$BIN_DIR/.grok-plus.new" <<EOF
#!/bin/sh
set -eu
UPDATER="$UPDATER"
BINARY="$CURRENT_LINK/grok-plus"
if [ "\${1:-}" = "update" ]; then
  shift
  if [ "\${1:-}" = "--force-reinstall" ]; then
    shift
    exec "\$UPDATER" --force "\$@"
  fi
  exec "\$UPDATER" "\$@"
fi
export GROK_DISABLE_AUTOUPDATER=1
exec "\$BINARY" "\$@"
EOF
chmod 755 "$BIN_DIR/.grok-plus.new"
/bin/mv -fh "$BIN_DIR/.grok-plus.new" "$COMMAND_PATH"

curl -fsSL "https://raw.githubusercontent.com/$REPO/main/install.sh" -o "$UPDATER.tmp"
chmod 755 "$UPDATER.tmp"
mv -f "$UPDATER.tmp" "$UPDATER"

log "Installed grok-plus $build_id"
log "Command: $COMMAND_PATH"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) log "Add $BIN_DIR to PATH if grok-plus is not found." ;;
esac
