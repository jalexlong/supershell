#!/bin/bash

# POINT THIS TO YOUR DEV BUILD
SUPERSHELL_BIN="$(pwd)/target/debug/supershell"

_supershell_hook() {
    # Fail-Open Check: Only run if the binary exists
    if [[ -x "$SUPERSHELL_BIN" ]]; then
        # Capture last command, stripping whitespace
        LAST_CMD=$(fc -ln -1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

        # Pass to Rust for evaluation
        "$SUPERSHELL_BIN" --check "$LAST_CMD"
    fi
}

# Attach the hook to the prompt command (runs after every command)
export PROMPT_COMMAND="_supershell_hook; $PROMPT_COMMAND"

# Create supershell alias, calling supershell as if it were on $PATH
alias supershell="$SUPERSHELL_BIN"

# --- THE CALL TO ACTION ---
# This prints once when the user sources the script.
echo " "
echo ">> [SYSTEM] SUPERSHELL DAEMON DETECTED."
echo ">> [ACTION] Type 'supershell' to initialize calibration..."
echo " "
