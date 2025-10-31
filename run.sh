#!/bin/bash

# Install the project if it isn't already installed
poetry install &> /dev/null

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
