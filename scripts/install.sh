#!/usr/bin/env sh

# exit on any error
set -eu

NAME="git-kit"
BIN=${BIN:-"/usr/local/bin"}

# colors
RED='\033[0;31m'
ORANGE='\033[0;33m'
NONE='\033[0m'

# logs
err() {
    echo "$1" >&2
}

error() {
    err "🙈 ${RED}error:${NONE} $1"
}

# utils
is_installed() {
    command -v "$1" 1>/dev/null 2>&1
}

derive_target_config() {
    plat=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "${plat}" in
        linux) echo "$HOME/.config/$NAME" ;;
        darwin) echo "$HOME/Library/Application Support/dev.xsv24.$NAME" ;;
        *) error "Currently unsupported OS platform '${plat}'" && exit 1 ;;
    esac
}

derive_zip_ext() {
    plat=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "${plat}" in
        *windows*) echo ".zip" ;;
        *) echo ".tar.gz"
    esac
}

derive_arch() {
    arch=$(uname -m | tr '[:upper:]' '[:lower:]')
    case "${arch}" in
        amd64 | x86_64) arch="x86_64" ;;
        armv*) arch="arm" ;;
        arm64) arch="aarch64" ;;
    esac

    # `uname -m` in some cases mis-reports 32-bit OS as 64-bit, so double check
    if [ "${arch}" = "x86_64" ] && [ "$(getconf LONG_BIT)" -eq 32 ]; then
        arch="i686"
    elif [ "${arch}" = "aarch64" ] && [ "$(getconf LONG_BIT)" -eq 32 ]; then
        arch="arm"
    fi

    echo "$arch"
}

derive_platform() {
    arch="$1"
    plat=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "${plat}-${arch}" in
        msys_nt*) echo "pc-windows-msvc" ;;
        cygwin_nt*) echo "pc-windows-msvc";;
        mingw*) echo "pc-windows-msvc" ;;
        darwin-*) echo "apple-darwin" ;;
        linux-arm) echo "unknown-linux-gnueabihf" ;;
        linux-*) echo "unknown-linux-musl" ;;
        freebsd-*) echo "unknown-freebsd" ;;
    esac
}

derive_binary_name() {
    arch=$(derive_arch)
    plat=$(derive_platform "$arch")

    echo "$NAME-$arch-$plat"
}

unzip() {
    printf "⏳ Unzipping binary... "

    path="$1"
    to="$2"
    ext=$(derive_zip_ext)

    case "$ext" in
    *.tar.gz)
        tar -xzof "$path" -C "$to"
        ;;
    *.zip)
        unzip -o "$path" -d "$to"
        ;;
    *)
        error "Unsupported compressed file type ${path}"
        exit 1
        ;;
    esac

    echo " ✅"
}

get_http_client() {
    if is_installed curl; then
        echo "curl --fail --silent --location --output"
    elif is_installed wget; then
        echo "wget --quiet --output-document="
    elif is_installed fetch; then
        echo "fetch --quiet --output="
    else
        error "Could not find http client please install one of the following:"
        err "→ curl, wget or fetch"
        exit 1
    fi
}

download() {
    file="$1"
    binary_name="$2"
    ext=$(derive_zip_ext)

    printf "⏳ Downlaoding binary %s... " "$binary_name"

    release="https://github.com/xsv24/$NAME/releases/latest/download/${binary_name}${ext}"

    request="$(get_http_client) $file $release"
    # execute request 
    $request && echo " ✅" && return 0

    echo ""
    error "Failed to download latest $NAME release for binary '${binary_name}'"
    exit 1
}

default_template_config() {
    binary_name="$1"
    uncompressed="$2"
    location="$uncompressed/$binary_name"
    
    printf "⏳ Configuring... "
    cp "$location/$NAME" "$BIN"

    target_config=$(derive_target_config)
    mkdir -p "$target_config"
    
    cp "$location/conventional.yml" "$target_config" 
    chmod 644 "$target_config/conventional.yml"

    cp "$location/default.yml" "$target_config"
    chmod 644 "$target_config/default.yml"

    echo " ✅"
}

main() {
    echo "⏳ Installing $NAME..."
    binary_name=$(derive_binary_name)
    compressed=$(mktemp)
    uncompressed=$(mktemp -d)

    download "$compressed" "$binary_name"
    unzip "$compressed" "$uncompressed"
    rm -r "$compressed"

    default_template_config "$binary_name" "$uncompressed"
    rm -r "$uncompressed"
    
    echo "🚀 ${ORANGE}$NAME${NONE} is now installed!"
    echo ""
    echo "✨ Get started with ↓"
    echo "> $NAME --help"
}

main
