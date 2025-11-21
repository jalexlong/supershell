#!/bin/bash

# ==============================================================================
# Supershell v0.2.0 - Bash Frontend Wrapper
#
# This script is the main entry point for the user. It is responsible for:
#   1. Starting and managing the Python background daemon.
#   2. Hooking into the Bash command lifecycle (`PROMPT_COMMAND`).
#   3. Sending command data to the daemon via the thin Python client.
#   4. Wrapping sensitive commands (`rm`, `mv`, etc.) for pre-execution checks.
# ==============================================================================

# --- Virtual Environment Setup ---
# Path to the virtual environment activation script
VENV_ACTIVATE_SCRIPT="./.venv/bin/activate"

# Check if the virtual environment exists and activate it
if [ -f "$VENV_ACTIVATE_SCRIPT" ]; then
    source "$VENV_ACTIVATE_SCRIPT"
    # Set the Python executable from the activated virtual environment
    PYTHON_EXEC="$VIRTUAL_ENV/bin/python3"
else
    echo "Error: Virtual environment not found. Please run 'uv venv' and 'uv pip install PyYAML rich' first."
    exit 1
fi

# Ensure the 'src' directory is in the PYTHONPATH so Python can find
# the 'supershell' package (e.g., supershell.daemon, supershell.client).
# This is crucial for 'python -m' to work correctly.
export PYTHONPATH="$PWD/src:$PYTHONPATH"

# --- Configuration ---
# Use a user-specific socket to allow for multiple users on the same system.
SOCKET_PATH="/tmp/supershell_${UID}.sock"
DAEMON_LOG="/tmp/supershell_daemon_${UID}.log"
DAEMON_PID_FILE="/tmp/supershell_daemon_${UID}.pid"


# --- Daemon Management ---

# Starts the Python daemon in the background.
start_daemon() {
    # Ensure no old daemon is running from a previous unclean exit.
    if [ -f "$DAEMON_PID_FILE" ]; then
        echo "Found stale PID file. Cleaning up..."
        kill "$(cat "$DAEMON_PID_FILE")" &>/dev/null
        rm -f "$DAEMON_PID_FILE" "$SOCKET_PATH"
    fi

    echo "Starting Supershell Daemon..."
    # Launch the daemon using the virtual environment's python.
    # 'nohup' prevents it from dying if the terminal closes unexpectedly.
    # All output is sent to a log file.
    nohup "$PYTHON_EXEC" -m supershell.daemon "$SOCKET_PATH" > "$DAEMON_LOG" 2>&1 &

    # Store the Process ID (PID) of the daemon.
    echo $! > "$DAEMON_PID_FILE"

    # --- Wait for Daemon to Create Socket (with timeout) ---
    local start_time=$(date +%s)
    local timeout=5 # seconds
    local waited_time=0

    echo -n "Waiting for daemon socket to appear..."
    while [ ! -S "$SOCKET_PATH" ] && [ "$waited_time" -lt "$timeout" ]; do
        sleep 0.1
        waited_time=$(( $(date +%s) - start_time ))
        echo -n "."
    done
    echo "" # Newline after dots

    if [ ! -S "$SOCKET_PATH" ]; then
        echo "Error: Daemon socket did not appear within $timeout seconds."
        echo "Check daemon log: $DAEMON_LOG"
        stop_daemon # Attempt to clean up
        exit 1
    fi
    # --- End Wait for Daemon ---

    echo "Daemon running with PID $(cat "$DAEMON_PID_FILE"). Log: $DAEMON_LOG"
}

# Stops the Python daemon and cleans up related files.
stop_daemon() {
    if [ -f "$DAEMON_PID_FILE" ]; then
        local PID
        PID=$(cat "$DAEMON_PID_FILE")
        echo -e "\nStopping Supershell Daemon (PID: $PID)..."
        kill "$PID" &>/dev/null # Use &>/dev/null to suppress "killed" message
        rm -f "$DAEMON_PID_FILE"
        rm -f "$SOCKET_PATH"
        echo "Cleanup complete. Goodbye."
    fi
}

# Ensure the daemon is stopped when the shell exits for any reason.
trap stop_daemon EXIT


# --- Bash Hooks ---

# This function is executed right after a command finishes and before the
# next prompt is displayed. It's our primary "post-execution" hook.
post_exec() {
    # Capture the exit code ($?) and the command string ($BASH_COMMAND) of the
    # last executed command.
    local last_exit_code=$?
    local last_command=$BASH_COMMAND

    # Trim leading/trailing whitespace using Bash parameter expansion
    # ${parameter##word} - Remove longest matching prefix pattern
    # ${parameter%%word} - Remove longest matching suffix pattern
    local trimmed_command="${last_command##*( )}" # Remove leading spaces
    trimmed_command="${trimmed_command%%*( )}" # Remove trailing spaces

    # For debugging: uncomment the next two lines to see what post_exec sees
    # echo "DEBUG: post_exec - last_command: '$last_command'" >> "$DAEMON_LOG"
    # echo "DEBUG: post_exec - trimmed_command: '$trimmed_command'" >> "$DAEMON_LOG"

    # Avoid triggering on empty commands or on the prompt command itself,
    # or on internal shell commands like 'source'.
    if [[ -n "$trimmed_command" && \
          "$trimmed_command" != "post_exec" && \
          "$trimmed_command" != "$PROMPT_COMMAND" && \
          "$trimmed_command" != "source supershell.sh" ]]; then
        # Use a subshell to call the client, preventing any environment
        # variable pollution in the user's active shell.
        (
            "$PYTHON_EXEC" -m supershell.client "$SOCKET_PATH" "$PWD" "post_exec" "$trimmed_command" "$last_exit_code"
        )
    fi
}
# Export the post_exec function so it's available to subshells
export -f post_exec


# --- Command Wrappers (for Pre-Execution Hooks) ---

# This is a generic handler that calls the daemon to ask for permission
# before executing a command.
_pre_exec_handler() {
    # Call the Python client with the "pre_exec" hook type.
    # The client will exit with 0 if the command is allowed, or non-zero to block.
    if "$PYTHON_EXEC" -m supershell.client "$SOCKET_PATH" "$PWD" "pre_exec" "$@"; then
        # Daemon gave permission. Execute the real command using 'command'.
        # 'command' ensures we call the actual binary, not our wrapper function.
        command "$@"
    else
        # Daemon blocked the command. The client is responsible for printing
        # any dialogue explaining why.
        return 1
    fi
}

# Override built-in commands by defining functions with the same name.
# These will be expanded first by Bash.

# Example wrapper for 'rm'
rm() {
    _pre_exec_handler "rm" "$@"
}

# Example wrapper for 'mv'
mv() {
    _pre_exec_handler "mv" "$@"
}

# Example wrapper for 'cp'
cp() {
    _pre_exec_handler "cp" "$@"
}


# --- Main Execution ---

# 1. Start the daemon process.
start_daemon

# 2. Set the PROMPT_COMMAND environment variable.
# Bash executes the content of this variable before displaying each prompt.
export PROMPT_COMMAND="post_exec"

# 3. Send the initial 'init' event to the daemon.
# This lets the game engine know the shell is ready and allows it to
# run any introductory scripts or dialogue.
"$PYTHON_EXEC" -m supershell.client "$SOCKET_PATH" "$PWD" "init"

# 4. Do NOT start a new interactive Bash session.
# By simply ending the script, the current shell becomes the Supershell.
# This ensures all functions and environment variables are inherited directly.

# The 'trap stop_daemon EXIT' command will automatically trigger when this
# interactive session ends (e.g., the user types 'exit').
