#!/bin/bash
set -e # Exit on error

APP_NAME="supershell"
INSTALL_DIR="$HOME/.local/bin"

# 1. DETECT OS & PATHS
# We must define DATA_DIR before we define HOOK_FILE
if [[ "$OSTYPE" == "darwin"* ]]; then
    DATA_DIR="$HOME/Library/Application Support/com.jalexlong.supershell"
else
    DATA_DIR="$HOME/.local/share/$APP_NAME"
fi

LIBRARY_DIR="$DATA_DIR/library"
HOOK_FILE="$DATA_DIR/init.sh"

# 2. PREPARE BINARY
# Check if we are in a "Pre-compiled" environment
if [ -f "./supershell" ]; then
    echo "📦 Found pre-compiled binary. Skipping build."
    SOURCE_BIN="./supershell"
elif [ -f "target/release/$APP_NAME" ]; then
    echo "🛠️  Using existing release build..."
    SOURCE_BIN="target/release/$APP_NAME"
else
    echo "🛠️  Building SuperShell (Release Mode)..."
    cargo build --release
    SOURCE_BIN="target/release/$APP_NAME"
fi

# 3. CREATE DIRECTORIES
echo "📂 Creating data directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$DATA_DIR"
mkdir -p "$LIBRARY_DIR"

# 4. INSTALL BINARY
echo "🚀 Installing binary to $INSTALL_DIR..."
cp "$SOURCE_BIN" "$INSTALL_DIR/$APP_NAME"
chmod +x "$INSTALL_DIR/$APP_NAME"

# 5. INSTALL ASSETS (The Library)
echo "📜 Installing Quest Library to $LIBRARY_DIR..."
# Clean old files to ensure no duplicates if we renamed things
rm -f "$LIBRARY_DIR"/*.yaml
# Copy all YAMLs from the local 'library' folder
cp -r library/*.yaml "$LIBRARY_DIR/"

# 6. GENERATE PRODUCTION HOOK
echo "🪝  Generating shell hook..."
cat <<EOF > "$HOOK_FILE"
#!/bin/bash

# PRODUCTION HOOK FOR SUPERSHELL
# Point to the installed binary
SUPERSHELL_BIN="$INSTALL_DIR/$APP_NAME"

_supershell_hook() {
    if [[ -x "\$SUPERSHELL_BIN" ]]; then
        local LAST_CMD=""

        # ZSH
        if [ -n "\$ZSH_VERSION" ]; then
             LAST_CMD=\$(fc -ln -1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        # BASH
        elif [ -n "\$BASH_VERSION" ]; then
             LAST_CMD=\$(fc -ln -1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        fi

        # EXECUTE
        if [[ -n "\$LAST_CMD" ]]; then
            "\$SUPERSHELL_BIN" --check "\$LAST_CMD"
        fi
    fi
}

# SETUP ZSH
if [ -n "\$ZSH_VERSION" ]; then
    autoload -Uz add-zsh-hook
    if [[ "\${precmd_functions[@]}" != *"_supershell_hook"* ]]; then
        add-zsh-hook precmd _supershell_hook
    fi

# SETUP BASH
elif [ -n "\$BASH_VERSION" ]; then
    if [[ ! "\$PROMPT_COMMAND" == *"_supershell_hook"* ]]; then
        export PROMPT_COMMAND="_supershell_hook; \$PROMPT_COMMAND"
    fi
fi

# ALIAS (Optional, ensures 'supershell' runs the specific binary)
alias supershell="\$SUPERSHELL_BIN"
EOF

# 7. UPDATE SHELL CONFIG
RC_FILE=""
if [ -n "$ZSH_VERSION" ]; then
    RC_FILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    if [ -f "$HOME/.bashrc" ]; then
        RC_FILE="$HOME/.bashrc"
    else
        RC_FILE="$HOME/.bash_profile"
    fi
fi

# Fallback detection
if [ -z "$RC_FILE" ]; then
    case "$SHELL" in
    */zsh) RC_FILE="$HOME/.zshrc" ;;
    */bash) RC_FILE="$HOME/.bashrc" ;;
    esac
fi

if [ -n "$RC_FILE" ]; then
    SOURCE_LINE="source \"$HOOK_FILE\""

    if grep -Fq "$HOOK_FILE" "$RC_FILE"; then
        echo "✅ Hook already present in $RC_FILE"
    else
        echo "✍️  Appending hook to $RC_FILE..."
        echo "" >> "$RC_FILE"
        echo "# SuperShell Hook" >> "$RC_FILE"
        echo "$SOURCE_LINE" >> "$RC_FILE"
    fi
else
    echo "⚠️  Could not detect shell config file. You must manually add:"
    echo "   source \"$HOOK_FILE\""
fi

# 8. PATH CHECK
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "⚠️  WARNING: $HOME/.local/bin is not in your \$PATH."
    echo "   You may need to add it to your shell config."
fi

echo "🎉 Installation Complete! Restart your terminal or run:"
echo "   source $RC_FILE"
