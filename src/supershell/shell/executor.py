import os
import subprocess
from dataclasses import dataclass

from supershell.tui.console import get_console


@dataclass
class CommandResult:
    command: str
    stdout: str
    stderr: str
    return_code: int


def execute_command(command: str) -> CommandResult:
    if command.startswith("cd "):
        try:
            parts = command.split(maxsplit=1)
            if len(parts) > 1:
                path = parts[1]
            else:
                path = "~"

            path = os.path.expanduser(path)
            os.chdir(path)
            return CommandResult(command=command, stdout="", stderr="", return_code=0)

        except FileNotFoundError:
            parts = command.split(maxsplit=1)
            path_str = parts[1] if len(parts) > 1 else "~"
            stderr = f"bash: cd: {path_str}: No such file or directory"
            return CommandResult(
                command=command, stdout="", stderr=stderr, return_code=1
            )
        except Exception as e:
            stderr = f"supershell error handling 'cd': {e}"
            return CommandResult(
                command=command, stdout="", stderr=stderr, return_code=1
            )

    if command.strip() == "clear":
        console = get_console()
        console.clear()
        return CommandResult(command=command, stdout="", stderr="", return_code=0)

    try:
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
        console = get_console()
        console.log(f"[danger]Fatal subprocess error: {e}[/danger]")
        return CommandResult(
            command=command,
            stdout="",
            stderr=f"supershell executor failure: {e}",
            return_code=127,
        )
