#!/bin/bash

echo "Starting supershell..."

# Install or update the project
if [[ ! -e "./poetry.lock" ]]; then
    poetry install &> /dev/null
else
    poetry update &> /dev/null
fi

# Start the game
poetry run supershell
