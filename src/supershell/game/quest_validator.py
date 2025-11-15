"""
Checks the result of a command against active quest objectives.
"""

import logging
import os
from typing import (  # Re-add Dict and List for clarity in type hints
    Any,
    Dict,
    List,
    Optional,
    Union,
)

from supershell.game import quest_manager
from supershell.shell.executor import CommandResult

log = logging.getLogger(__name__)


def check(command_str: str, command_result: CommandResult) -> str | None:
    """
    Checks the command result against the active objective.
    If complete, returns the objective's ID.
    Otherwise, returns None.
    """
    # Explicitly mark command_str as unused for linting purposes for now
    _ = command_str

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

        elif active_obj.type == "multi_path_exists":  # NEW OBJECTIVE TYPE
            is_complete = _check_multi_path_exists(active_obj, command_result)

        # --- Return the ID if complete ---
        if is_complete:
            log.info(f"Objective complete: {active_obj.id}")
            quest_manager.mark_objective_complete(active_obj.id)
            return active_obj.id  # This is the "event"

    except Exception as e:
        log.error(f"Error during objective check for {active_obj.id}: {e}")

    return None  # Not complete


# --- HELPER FOR CRITERIA TYPE CHECK ---
def _get_criteria_dict(obj: quest_manager.Objective) -> Optional[Dict[str, Any]]:
    if not isinstance(obj.criteria, dict):
        log.error(
            f"Objective {obj.id} has type '{obj.type}' but criteria is not a dictionary."
        )
        return None
    return obj.criteria


# --- CHECKER FUNCTIONS ---


def _check_command_run(obj: quest_manager.Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    if res.return_code != 0:
        return False
    command_ran = res.command.strip().split()[0].lower()
    expected_command = criteria.get("command")
    return command_ran == expected_command


def _check_path_exists(obj: quest_manager.Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    path_to_check = criteria.get("path")
    expected_type = criteria.get("type")
    if not path_to_check:
        return False

    full_path = os.path.expanduser(path_to_check)
    if expected_type == "dir":
        return os.path.isdir(full_path)
    elif expected_type == "file":
        return os.path.isfile(full_path)
    return False


def _check_any_command(obj: quest_manager.Objective, res: CommandResult) -> bool:
    # 'any_command' doesn't have specific dict criteria, but it's good to be explicit
    # and ensures criteria isn't a list if type was mismatched.
    criteria = _get_criteria_dict(obj)
    if criteria is None:  # Still check to ensure it's not a list
        return False
    return res.return_code == 0


def _check_cwd_is(obj: quest_manager.Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    path_to_check = criteria.get("path")
    if not path_to_check:
        return False
    return os.getcwd() == os.path.expanduser(path_to_check)


def _check_path_not_exists(obj: quest_manager.Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    path_to_check = criteria.get("path")
    if not path_to_check:
        return False
    return not os.path.exists(os.path.expanduser(path_to_check))


def _check_file_contains(obj: quest_manager.Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    path_to_check = os.path.expanduser(str(criteria.get("path", "")))

    content_to_find = criteria.get("content")
    content_from_save_key = criteria.get("content_from_save")

    if content_from_save_key:
        save_data = quest_manager.get_save_data()
        content_to_find = save_data.get(content_from_save_key)

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
    criteria = _get_criteria_dict(obj)
    if criteria is None:  # Still check to ensure it's not a list
        return False
    """This check can *only* be completed by a game command."""
    return False


def _check_multi_path_exists(obj: quest_manager.Objective, res: CommandResult) -> bool:
    # For 'multi_path_exists', criteria is expected to be a list
    if not isinstance(obj.criteria, list):
        log.error(
            f"Objective {obj.id} has type 'multi_path_exists' but criteria is not a list."
        )
        return False

    for item in obj.criteria:  # Iterate directly over the list
        path_to_check = item.get("path")
        expected_type = item.get("type")
        should_exist = item.get("exist", True)  # Default to True if 'exist' is missing

        if not path_to_check:
            log.warning(f"Missing 'path' in multi_path_exists criteria for {obj.id}")
            return False

        full_path = os.path.expanduser(path_to_check)

        # Check based on expected_type first
        exists = False
        if expected_type == "dir":
            exists = os.path.isdir(full_path)
        elif expected_type == "file":
            exists = os.path.isfile(full_path)
        else:  # Default to generic existence check if type is not specified or recognized
            exists = os.path.exists(full_path)

        if should_exist and not exists:
            log.debug(f"Path {full_path} should exist, but does not.")
            return False
        if not should_exist and exists:
            log.debug(f"Path {full_path} should NOT exist, but does.")
            return False

    # All criteria passed
    return True
