"""
Checks the result of a command against active quest objectives.
"""

import logging
import os
from typing import (  # Re-add Dict and List for clarity in type hints
    Any,
    Callable,
    Dict,
    Optional,
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

        # --- Route to the correct checker function using the dispatch map ---
        checker_func = _CHECKER_MAP.get(active_obj.type)
        if checker_func:
            is_complete = checker_func(active_obj, command_result)
        else:
            log.warning(
                f"No checker function found for objective type: {active_obj.type}"
            )

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


def _check_checklist(obj: quest_manager.Objective, res: CommandResult) -> bool:
    """
    Checks a list of sub-objectives.
    Fails if *any* sub-check fails.
    """
    # 1. Validate that the criteria is a list
    if not isinstance(obj.criteria, list):
        log.error(
            f"Objective {obj.id} has type 'checklist' but criteria is not a list."
        )
        return False

    criteria_list = obj.criteria

    # 2. Iterate through each sub-check defined in the list
    for sub_check_data in criteria_list:
        sub_type = sub_check_data.get("type")
        sub_criteria = sub_check_data.get("criteria")

        if not sub_type or sub_criteria is None:
            log.warning(f"Checklist item for {obj.id} is missing 'type' or 'criteria'.")
            return False  # Fail the whole check

        # 3. Find the checker function for the sub_type
        checker_function = _CHECKER_MAP.get(sub_type)

        if not checker_function:
            log.warning(
                f"Checklist for {obj.id}: Unknown sub-objective type: {sub_type}"
            )
            return False  # Fail the whole check

        if checker_function == _check_checklist:
            log.error(f"Checklist {obj.id}: Nested checklists are not allowed.")
            return False  # Prevent recursion

        # 4. Create a temporary "fake" Objective object for the sub-check.
        # This is necessary because our other checkers expect an 'Objective'
        # object, not just a criteria dict.
        try:
            temp_obj_data = {
                "id": f"{obj.id}_sub_{sub_type}",
                "type": sub_type,
                "criteria": sub_criteria,
            }
            temp_obj = quest_manager.Objective.from_dict(temp_obj_data)
        except Exception as e:
            log.error(f"Failed to create temp_obj for checklist: {e}", exc_info=True)
            return False

        # 5. Run the sub-check. If any check fails, the whole list fails.
        if not checker_function(temp_obj, res):
            log.debug(f"Checklist sub-check failed: {sub_type} with {sub_criteria}")
            return False

    # 6. If the loop finishes, all sub-checks passed
    return True


_CHECKER_MAP: Dict[str, Callable[[quest_manager.Objective, CommandResult], bool]] = {
    "command_run": _check_command_run,
    "path_exists": _check_path_exists,
    "path_not_exists": _check_path_not_exists,
    "file_contains": _check_file_contains,
    "any_command": _check_any_command,
    "cwd_is": _check_cwd_is,
    "manual_complete": _check_manual_complete,
    "checklist": _check_checklist,
}
