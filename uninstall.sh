#!/bin/bash
set -e

APP_NAME="supershell"
INSTALL_DIR="$HOME/.local/bin"
BINARY_PATH="$INSTALL_DIR/$APP_NAME"

# DETECT OS (To find the correct Data Directory to delete)
if [[ "$OSTYPE" == "darwin"* ]]; then
    DATA_DIR="$HOME/Library/Application Support/com.jalexlong.supershell"
else
    DATA_DIR="$HOME/.local/share/$APP_NAME"
fi

HOOK_FILE="$DATA_DIR/init.sh"

echo "🗑️  Uninstalling SuperShell..."

# 1. Remove Binary
if [ -f "$BINARY_PATH" ]; then
    rm "$BINARY_PATH"
    echo "✅ Removed binary: $BINARY_PATH"
else
    echo "⚠️  Binary not found at $BINARY_PATH (skipping)"
fi

# 2. Remove Data Directory (Quests, Saves, Hook, Library)
if [ -d "$DATA_DIR" ]; then
    rm -rf "$DATA_DIR"
    echo "✅ Removed data directory: $DATA_DIR"
else
    echo "⚠️  Data directory not found at $DATA_DIR (skipping)"
fi

# 3. Instructions for Shell Config
echo ""
echo "========================================================"
echo "⚠️  ACTION REQUIRED: CLEAN UP YOUR SHELL CONFIG"
echo "========================================================"
echo "The installer added a 'source' line to your configuration file."
echo "You must manually remove it to prevent terminal errors."
echo ""
echo "1. Open your config file:"
echo "   nano ~/.bashrc   (or ~/.zshrc)"
echo ""
echo "2. Find and delete these lines:"
echo "   # SuperShell Hook"
echo "   source \"$HOOK_FILE\""
echo ""
echo "3. Save and close."
echo "========================================================"
echo "Uninstallation complete."
