#!/usr/bin/env sh
set -eu

REPO="${CHANGEGUARD_REPO:-UnlikelyKiller/ChangeGuard}"
VERSION="${CHANGEGUARD_VERSION:-latest}"
INSTALL_DIR="${CHANGEGUARD_INSTALL_DIR:-$HOME/.local}"
NO_PATH_UPDATE="${CHANGEGUARD_NO_PATH_UPDATE:-0}"
BUILD_FROM_SOURCE="${CHANGEGUARD_BUILD_FROM_SOURCE:-0}"
FEATURES=""

if [ "${CHANGEGUARD_DAEMON:-0}" = "1" ]; then
  FEATURES="--features daemon"
fi

step() {
  printf '==> %s\n' "$1"
}

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

install_from_release() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Linux:x86_64) asset="changeguard-x86_64-unknown-linux-gnu.tar.gz" ;;
    Darwin:x86_64) asset="changeguard-x86_64-apple-darwin.tar.gz" ;;
    Darwin:arm64) asset="changeguard-aarch64-apple-darwin.tar.gz" ;;
    *) return 1 ;;
  esac

  tag_path="latest/download"
  if [ "$VERSION" != "latest" ]; then
    tag_path="download/$VERSION"
  fi

  url="https://github.com/$REPO/releases/$tag_path/$asset"
  tmp_dir="$(mktemp -d)"
  archive="$tmp_dir/$asset"
  mkdir -p "$INSTALL_DIR/bin"

  step "Downloading $url"
  if has_cmd curl; then
    curl -fsSL "$url" -o "$archive"
  elif has_cmd wget; then
    wget -q "$url" -O "$archive"
  else
    return 1
  fi

  tar -xzf "$archive" -C "$tmp_dir"
  bin_path="$(find "$tmp_dir" -type f -name changeguard | head -n 1)"
  if [ -z "$bin_path" ]; then
    return 1
  fi

  cp "$bin_path" "$INSTALL_DIR/bin/changeguard"
  chmod +x "$INSTALL_DIR/bin/changeguard"
  rm -rf "$tmp_dir"
}

install_from_cargo() {
  if ! has_cmd cargo; then
    echo "Rust cargo was not found. Install Rust from https://rustup.rs or publish a ChangeGuard release asset, then rerun this installer." >&2
    exit 1
  fi

  if [ -f Cargo.toml ] && grep -q 'name = "changeguard"' Cargo.toml; then
    step "Installing ChangeGuard from the current checkout"
    # shellcheck disable=SC2086
    cargo install --path . --locked --root "$INSTALL_DIR" $FEATURES
  else
    step "Installing ChangeGuard from https://github.com/$REPO"
    # shellcheck disable=SC2086
    cargo install --git "https://github.com/$REPO" --branch main --locked --root "$INSTALL_DIR" $FEATURES
  fi
}

if [ "$BUILD_FROM_SOURCE" = "1" ]; then
  install_from_cargo
else
  if ! install_from_release; then
    step "Release install failed; falling back to cargo install"
    install_from_cargo
  fi
fi

if [ "$NO_PATH_UPDATE" != "1" ]; then
  case ":$PATH:" in
    *":$INSTALL_DIR/bin:"*) ;;
    *)
      shell_rc=""
      if [ -n "${SHELL:-}" ]; then
        case "$SHELL" in
          */zsh) shell_rc="$HOME/.zshrc" ;;
          */bash) shell_rc="$HOME/.bashrc" ;;
        esac
      fi
      if [ -n "$shell_rc" ]; then
        touch "$shell_rc"
        if ! grep -q "$INSTALL_DIR/bin" "$shell_rc"; then
          printf '\nexport PATH="$PATH:%s/bin"\n' "$INSTALL_DIR" >> "$shell_rc"
          step "Added $INSTALL_DIR/bin to $shell_rc. Open a new terminal for other sessions."
        fi
      else
        step "Add $INSTALL_DIR/bin to PATH for future shells."
      fi
      export PATH="$PATH:$INSTALL_DIR/bin"
      ;;
  esac
fi

step "Verifying installation"
"$INSTALL_DIR/bin/changeguard" --help | sed -n '1,5p'

printf '\nChangeGuard installed. AI agents can now run: changeguard doctor\n'
