#!/bin/bash

# POINT THIS TO YOUR DEV BUILD
SUPERSHELL_BIN="$(pwd)/target/debug/supershell"

_supershell_hook() {
    if [[ -x "$SUPERSHELL_BIN" ]]; then
        local LAST_CMD=""

	# Extract history differently based on Shell
	if [ -n "$ZSH_VERSION" ]; then
	    # ZSH: `fc j-ln -1` gets the last command (no timestamps)
	    # sed trims leading/trailing whitespace
	    LAST_CMD=$(fc -ln -1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
	elif [ -n "$BASH_VERSION" ]; then
	    # BASH: `fc -ln -1` works similarly
            LAST_CMD=$(fc -ln -1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
	fi

        # Pass to Rust for evaluation
	if [[ -n "$LAST_CMD" ]]; then
	    "$SUPERSHELL_BIN" --check "$LAST_CMD"
	fi
    fi
}

if [ -n "$ZSH_VERSION" ]; then
    # --- ZSH SETUP ---
    autoload -Uz add-zsh-hook
    
    # Check if hook is already added to avoid duplication
    if [[ "${precmd_functions[@]}" != *"_supershell_hook"* ]]; then
        add-zsh-hook precmd _supershell_hook
    fi

elif [ -n "$BASH_VERSION" ]; then
    # --- BASH SETUP ---
    # Attach to PROMPT_COMMAND if not already present
    if [[ ! "$PROMPT_COMMAND" == *"_supershell_hook"* ]]; then
        export PROMPT_COMMAND="_supershell_hook; $PROMPT_COMMAND"
    fi
fi

# Create supershell alias, calling supershell as if it were on $PATH
alias supershell="$SUPERSHELL_BIN"

echo ">> SuperShell hook loaded for: $(ps -p $$ -o comm=)"

