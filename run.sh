#!/bin/bash

echo "Starting supershell..."

# Install or update the project
if [[ ! -e "./poetry.lock" ]]; then
    poetry install &> /dev/null
else
    poetry update &> /dev/null
fi

# Activate the virtual environment
eval "$(poetry env activate)"

# Save the directory the user is running the script from
ORIGINAL_DIR="$pwd"

# Go $HOME
cd ~

# Start the game
supershell

# When done, go back to the original directory
cd $ORIGINAL_DIR
