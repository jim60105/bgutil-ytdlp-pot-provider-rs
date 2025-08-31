#!/bin/bash

# Usage: ./install_plugin_dev.sh [--add-conf]
# If --add-conf is provided, yt-dlp.conf will be created/modified.

ADD_CONF=false
for arg in "$@"; do
    if [ "$arg" = "--add-conf" ]; then
        ADD_CONF=true
    elif [ -n "$arg" ]; then
        echo "ERROR: Unrecognized argument: $arg" >&2
        exit 1
    fi
done


PLUGIN_DIR=~/yt-dlp-plugins/bgutil-ytdlp-pot-provider/
rm -rf "$PLUGIN_DIR"
mkdir -p "$PLUGIN_DIR"
echo "Installing plugin to $PLUGIN_DIR"
cp -r plugin/* "$PLUGIN_DIR"


if [ "$ADD_CONF" = true ]; then
    YTDLP_CONF=~/yt-dlp.conf
    if [ -e "$YTDLP_CONF" ]; then
        echo "WARN: yt-dlp.conf already exists at $YTDLP_CONF. Delete it if you want to recreate it." >&2
    else
        echo "Adding yt-dlp configuration to $YTDLP_CONF"
        echo -e "--extractor-args \"youtubepot-bgutilscript:script_path=$(realpath target/debug/bgutil-pot-generate)\"" > "$YTDLP_CONF"
        echo -e '--extractor-args "youtube:player-client=mweb"' >> "$YTDLP_CONF"
    fi
else
    echo "WARN: yt-dlp.conf was not created. To add it, run: $0 --add-conf"
fi

echo "Building Rust binaries"
cargo build
echo "DONE!"
echo -e "Use the following commands:"
echo -e "  Start the server: ./target/debug/bgutil-pot-server"
echo -e "  Generate single token: ./target/debug/bgutil-pot-generate"
