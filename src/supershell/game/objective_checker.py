"""
Checks the result of a command against active quest objectives.
"""

import logging
import os

from supershell.game import quest_manager
from supershell.shell.executor import CommandResult

log = logging.getLogger(__name__)


def check(command_result: CommandResult) -> str | None:
    """
    Checks the command result against the active objective.
    If complete, returns the objective's ID.
    Otherwise, returns None.
    """
    active_obj = quest_manager.get_active_objective()
    if not active_obj:
        return None  # No active quest

    log.debug(
        f"Checking {command_result.command} against {active_obj.id} ({active_obj.type})"
    )

    try:
        is_complete = False

        # --- Route to the correct checker function ---
        if active_obj.type == "command_run":
            is_complete = _check_command_run(active_obj, command_result)

        elif active_obj.type == "path_exists":
            is_complete = _check_path_exists(active_obj, command_result)

        elif active_obj.type == "file_contains":
            is_complete = _check_file_contains(active_obj, command_result)

        elif active_obj.type == "any_command":
            is_complete = _check_any_command(active_obj, command_result)

        elif active_obj.type == "cwd_is":
            is_complete = _check_cwd_is(active_obj, command_result)

        elif active_obj.type == "path_not_exists":
            is_complete = _check_path_not_exists(active_obj, command_result)

        elif active_obj.type == "manual_complete":
            is_complete = _check_manual_complete(active_obj, command_result)

        # --- Return the ID if complete ---
        if is_complete:
            log.info(f"Objective complete: {active_obj.id}")
            quest_manager.mark_objective_complete(active_obj.id)
            return active_obj.id  # This is the "event"

    except Exception as e:
        log.error(f"Error during objective check for {active_obj.id}: {e}")

    return None  # Not complete


# --- CHECKER FUNCTIONS ---


def _check_command_run(obj: quest_manager.Objective, res: CommandResult) -> bool:
    if res.return_code != 0:
        return False
    command_ran = res.command.strip().split()[0].lower()
    expected_command = obj.criteria.get("command")
    return command_ran == expected_command


def _check_path_exists(obj: quest_manager.Objective, res: CommandResult) -> bool:
    path_to_check = obj.criteria.get("path")
    expected_type = obj.criteria.get("type")
    if not path_to_check:
        return False

    full_path = os.path.expanduser(path_to_check)
    if expected_type == "dir":
        return os.path.isdir(full_path)
    elif expected_type == "file":
        return os.path.isfile(full_path)
    return False


def _check_any_command(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """Checks if *any* command was run successfully."""
    return res.return_code == 0


def _check_cwd_is(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """Checks if the player is in a specific directory."""
    path_to_check = obj.criteria.get("path")
    if not path_to_check:
        return False
    return os.getcwd() == os.path.expanduser(path_to_check)


def _check_path_not_exists(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """Checks if a file or directory is GONE."""
    path_to_check = obj.criteria.get("path")
    if not path_to_check:
        return False
    return not os.path.exists(os.path.expanduser(path_to_check))


def _check_file_contains(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """Checks if a file (or any file in a dir) contains text."""
    path_to_check = os.path.expanduser(str(obj.criteria.get("path")))
    content_to_find = obj.criteria.get("content")
    if not path_to_check or not content_to_find:
        return False

    if os.path.isfile(path_to_check):
        try:
            with open(path_to_check, "r") as f:
                return content_to_find in f.read()
        except (IOError, OSError):
            return False

    elif os.path.isdir(path_to_check):
        # Path is a directory. Search *all* files inside it.
        for root, dirs, files in os.walk(path_to_check):
            for file in files:
                try:
                    with open(os.path.join(root, file), "r") as f:
                        if content_to_find in f.read():
                            return True
                except (IOError, OSError):
                    continue
        return False
    return False


def _check_manual_complete(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """This check can *only* be completed by a game command."""
    return False
