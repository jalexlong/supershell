"""
Handles the "Eval" part of the REPL.

Executes commands in a real bash shell and captures the output.
"""

import os
import subprocess
from dataclasses import dataclass

from supershell.tui.console import get_console


@dataclass
class CommandResult:
    """A simple data structure to hold the results of an executed command."""

    command: str
    stdout: str
    stderr: str
    return_code: int


def execute_command(command: str) -> CommandResult:
    """
    Executes a command in the /bin/bash shell.
    """

    # --- 1. Special Case: `cd` command ---
    # This *must* be handled by Python, otherwise, the subprocess
    # will change its own directory and then exit, leaving the
    # main script's directory unchanged.
    if command.startswith("cd "):
        try:
            # Get the target directory, handling 'cd' with no args
            parts = command.split(maxsplit=1)
            if len(parts) > 1:
                path = parts[1]
            else:
                path = "~"  # 'cd' by itself goes home

            # Expand user-agnostic paths like '~/'
            path = os.path.expanduser(path)

            # Change the directory *of the Python process*
            os.chdir(path)

            # Return a "fake" successful result
            return CommandResult(command=command, stdout="", stderr="", return_code=0)

        except FileNotFoundError:
            parts = command.split(maxsplit=1)
            path = parts[1]

            stderr = f"bash: cd: {path}: No such file or directory"
            return CommandResult(
                command=command, stdout="", stderr=stderr, return_code=1
            )
        except Exception as e:
            stderr = f"supershell error handling 'cd': {e}"
            return CommandResult(
                command=command, stdout="", stderr=stderr, return_code=1
            )

    # --- 2. Special Case: `clear` command ---
    # Running the `clear` command typically submits the ANSI escape
    # sequence `[H[J[3J`. We intercept the command and use Rich's own
    # `.clear()` method and return a fake success instead.
    if command.strip() == "clear":
        console = get_console()
        console.clear()  # Use Rich's built-in clear method
        return CommandResult(command=command, stdout="", stderr="", return_code=0)

    # --- 3. All Other Commands ---
    try:
        # `shell=True` runs the command through the shell, allowing
        # pipes, redirection, and bash logic.
        # `executable='/bin/bash'` ensures we're using bash.
        result = subprocess.run(
            command, shell=True, capture_output=True, text=True, executable="/bin/bash"
        )

        return CommandResult(
            command=command,
            stdout=result.stdout.strip(),
            stderr=result.stderr.strip(),
            return_code=result.returncode,
        )

    except Exception as e:
        # Catch any unexpected errors during subprocess execution
        console = get_console()
        console.log(f"[danger]Fatal subprocess error: {e}[/danger]")
        return CommandResult(
            command=command,
            stdout="",
            stderr=f"supershell executor failure: {e}",
            return_code=127,  # Command not found
        )
