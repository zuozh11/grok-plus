#!/bin/sh
set -eu

REPO="${GROK_PLUS_REPO:-zuozh11/grok-plus}"
HOME_DIR="${GROK_PLUS_HOME:-$HOME/.local/share/grok-plus}"
BIN_DIR="${GROK_PLUS_BIN_DIR:-$HOME/.local/bin}"
RELEASES_DIR="$HOME_DIR/releases"
CURRENT_LINK="$HOME_DIR/current"
COMMAND_LINK="$BIN_DIR/grok-plus"
UPDATER="$HOME_DIR/update"
LAUNCH_AGENT="$HOME/Library/LaunchAgents/com.zuozhi.grok-plus.update.plist"
ASSET="grok-plus-aarch64-apple-darwin.tar.gz"

force=0
check=0
quiet=0
install_agent=1
requested_version=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --force) force=1 ;;
    --check) check=1 ;;
    --quiet) quiet=1 ;;
    --no-launch-agent) install_agent=0 ;;
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

if [ -n "$requested_version" ]; then
  tag="v${requested_version#v}"
  base_url="https://github.com/$REPO/releases/download/$tag"
else
  base_url="https://github.com/$REPO/releases/latest/download"
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/grok-plus.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

archive="$tmp_dir/$ASSET"
checksum="$tmp_dir/$ASSET.sha256"
curl -fsSL "$base_url/$ASSET" -o "$archive"
curl -fsSL "$base_url/$ASSET.sha256" -o "$checksum"
(cd "$tmp_dir" && shasum -a 256 -c "$ASSET.sha256" >/dev/null)
tar -xzf "$archive" -C "$tmp_dir"

version="$(cat "$tmp_dir/VERSION")"
current_version=""
if [ -f "$CURRENT_LINK/VERSION" ]; then
  current_version="$(cat "$CURRENT_LINK/VERSION")"
fi

if [ "$check" -eq 1 ]; then
  if [ "$current_version" = "$version" ]; then
    log "grok-plus is up to date ($version)."
    exit 0
  fi
  log "grok-plus update available: ${current_version:-not installed} -> $version"
  exit 0
fi

mkdir -p "$RELEASES_DIR" "$BIN_DIR" "$HOME_DIR"
release_dir="$RELEASES_DIR/$version"
if [ "$force" -eq 1 ] || [ ! -x "$release_dir/grok-plus" ]; then
  stage="$RELEASES_DIR/.${version}.tmp.$$"
  rm -rf "$stage"
  mkdir -p "$stage"
  cp "$tmp_dir/grok-plus" "$stage/grok-plus"
  cp "$tmp_dir/VERSION" "$stage/VERSION"
  chmod 755 "$stage/grok-plus"
  rm -rf "$release_dir"
  mv "$stage" "$release_dir"
fi

ln -sfn "$release_dir" "$HOME_DIR/.current.new"
mv -f "$HOME_DIR/.current.new" "$CURRENT_LINK"
ln -sfn "$CURRENT_LINK/grok-plus" "$BIN_DIR/.grok-plus.new"
mv -f "$BIN_DIR/.grok-plus.new" "$COMMAND_LINK"

curl -fsSL "https://raw.githubusercontent.com/$REPO/main/install.sh" -o "$UPDATER.tmp"
chmod 755 "$UPDATER.tmp"
mv -f "$UPDATER.tmp" "$UPDATER"

if [ "$install_agent" -eq 1 ]; then
  mkdir -p "$(dirname "$LAUNCH_AGENT")"
  cat > "$LAUNCH_AGENT" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.zuozhi.grok-plus.update</string>
  <key>ProgramArguments</key>
  <array>
    <string>$UPDATER</string>
    <string>--quiet</string>
    <string>--no-launch-agent</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>StartInterval</key>
  <integer>21600</integer>
  <key>StandardOutPath</key>
  <string>$HOME_DIR/update.log</string>
  <key>StandardErrorPath</key>
  <string>$HOME_DIR/update.log</string>
</dict>
</plist>
EOF
  launchctl bootout "gui/$(id -u)" "$LAUNCH_AGENT" >/dev/null 2>&1 || true
  launchctl bootstrap "gui/$(id -u)" "$LAUNCH_AGENT"
fi

log "Installed grok-plus $version"
log "Command: $COMMAND_LINK"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) log "Add $BIN_DIR to PATH if grok-plus is not found." ;;
esac
