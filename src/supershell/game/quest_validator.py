"""
Validates command results against active quest objectives.

This module is responsible for the core logic of determining if a user's
action resulted in the completion or failure of a quest objective. It is
designed to be stateless and rely only on the provided command result and
the current state managed by `quest_manager`.

The primary function, `check`, returns a tri-state status:
- SUCCESS: The objective's completion criteria were met.
- FAIL: The objective's failure criteria were met.
- CONTINUE: Neither success nor failure criteria were met; no state change.
"""

import logging
import os
from typing import (
    Any,
    Callable,
    Dict,
    Literal,
    Optional,
    Tuple,
)

from supershell.game import quest_manager
from supershell.game.models import CommandResult, Objective

log = logging.getLogger(__name__)

# --- Type Definitions ---
ValidationStatus = Literal["SUCCESS", "FAIL", "CONTINUE"]


# --- Main Validator Function ---


def check(command_result: CommandResult) -> Tuple[ValidationStatus, Optional[str]]:
    """
    Checks a command result against the active objective's success and failure
    conditions.

    Args:
        command_result: A CommandResult object containing the command string
                        and its return code.

    Returns:
        A tuple containing the validation status (`SUCCESS`, `FAIL`, or
        `CONTINUE`) and the ID of the objective that was checked.
    """
    active_obj = quest_manager.get_active_objective()
    if not active_obj:
        return "CONTINUE", None

    log.debug(
        f"Validating command '{command_result.command}' against objective '{active_obj.id}'"
    )

    # 1. Check for explicit failure conditions first.
    # These take precedence over success conditions.
    if active_obj.fail_type and active_obj.fail_criteria:
        fail_checker_func = _CHECKER_MAP.get(active_obj.fail_type)
        if fail_checker_func:
            # Create a temporary "objective" to represent the failure condition
            # so we can reuse the standard checker functions.
            temp_fail_obj_data: Dict[str, Any] = {
                "id": f"{active_obj.id}_fail_check",
                "type": active_obj.fail_type,
                "criteria": active_obj.fail_criteria,
                "hint": "",  # Temp objectives don't need hints
                "description": "",  # Temp objectives don't need descriptions
                # Explicitly set optional fields to None if they're not relevant for the temp objective
                "on_complete_script": [],
                "on_command_fail_script": [],
                "required_cwd": None,
                "fail_type": None,  # A fail check shouldn't have nested fail_type/criteria
                "fail_criteria": None,
            }
            temp_fail_obj = Objective.from_dict(temp_fail_obj_data)

            if fail_checker_func(temp_fail_obj, command_result):
                log.info(
                    f"Objective '{active_obj.id}' FAIL condition met: {active_obj.fail_type}"
                )
                return "FAIL", active_obj.id

    # 2. If no failure, check for success conditions.
    success_checker_func = _CHECKER_MAP.get(active_obj.type)
    if success_checker_func:
        if success_checker_func(active_obj, command_result):
            log.info(f"Objective '{active_obj.id}' SUCCESS condition met.")
            return "SUCCESS", active_obj.id

    # 3. If neither success nor failure, the game state continues as is.
    return "CONTINUE", None


# --- Helper for Criteria Type Check ---


def _get_criteria_dict(obj: Objective) -> Optional[Dict[str, Any]]:
    """Ensures that the criteria for an objective is a dictionary."""
    if not isinstance(obj.criteria, dict):
        log.error(
            f"Objective '{obj.id}' has type '{obj.type}' but its criteria is not a dictionary."
        )
        return None
    return obj.criteria


# --- Individual Checker Functions ---
# Each function checks a specific condition and returns True if met, False otherwise.
# CRITICAL: None of these functions should inspect `res.stdout` or `res.stderr`.


def _check_command_run(obj: Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None or res.return_code != 0:
        return False

    command_ran = res.command.strip().split()[0].lower()
    expected_command = criteria.get("command")
    return command_ran == expected_command


def _check_path_exists(obj: Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    path_to_check = criteria.get("path")
    expected_type = criteria.get("type")  # 'file' or 'dir'
    if not path_to_check:
        return False

    full_path = os.path.expanduser(path_to_check)
    if expected_type == "dir":
        return os.path.isdir(full_path)
    if expected_type == "file":
        return os.path.isfile(full_path)
    # If type is not specified, just check for existence
    return os.path.exists(full_path)


def _check_any_command(obj: Objective, res: CommandResult) -> bool:
    return res.return_code == 0


def _check_cwd_is(obj: Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    path_to_check = criteria.get("path")
    if not path_to_check:
        return False
    return os.getcwd() == os.path.expanduser(path_to_check)


def _check_path_not_exists(obj: Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    path_to_check = criteria.get("path")
    if not path_to_check:
        return False
    return not os.path.exists(os.path.expanduser(path_to_check))


def _check_file_contains(obj: Objective, res: CommandResult) -> bool:
    criteria = _get_criteria_dict(obj)
    if criteria is None:
        return False

    path_to_check = os.path.expanduser(str(criteria.get("path", "")))
    content_to_find = criteria.get("content")

    if not path_to_check or not content_to_find:
        return False

    if not os.path.isfile(path_to_check):
        return False

    try:
        with open(path_to_check, "r") as f:
            return content_to_find in f.read()
    except (IOError, OSError):
        return False


def _check_manual_complete(obj: Objective, res: CommandResult) -> bool:
    """This check can only be completed by a game command, not a shell command."""
    return False


def _check_checklist(obj: Objective, res: CommandResult) -> bool:
    """Checks a list of sub-objectives. Succeeds only if all sub-checks pass."""
    if not isinstance(obj.criteria, list):
        log.error(
            f"Objective '{obj.id}' has type 'checklist' but criteria is not a list."
        )
        return False

    for sub_check_data in obj.criteria:
        sub_type_raw = sub_check_data.get("type")
        sub_criteria = sub_check_data.get("criteria")

        if not isinstance(sub_type_raw, str):
            log.warning(f"Checklist item for '{obj.id}' has invalid or missing 'type'.")
            return False

        sub_type: str = sub_type_raw
        checker_func = _CHECKER_MAP.get(sub_type)

        if not checker_func:
            log.warning(f"Checklist for '{obj.id}': Unknown sub-type '{sub_type}'")
            return False  # An unknown sub-type causes the whole checklist to fail.

        # Ensure sub_criteria is a dictionary or appropriate type for the sub_type
        # If it's expected to be a dict, check it. If it's for e.g., any_command, it might be None.
        if sub_criteria is not None and not isinstance(sub_criteria, (dict, list)):
            log.warning(
                f"Checklist item for '{obj.id}' has invalid 'criteria' for type '{sub_type}'. It should be a dict or list (if applicable to sub_type)."
            )
            return False

        # Create a temporary Objective object to pass to the sub-checker.
        temp_obj_data: Dict[str, Any] = {
            "id": f"{obj.id}_sub_{sub_type}",
            "type": sub_type,
            "criteria": sub_criteria,
            "hint": "",  # Temp objectives don't need hints
            "description": "",  # Temp objectives don't need descriptions
            # Explicitly setting optional fields to None if they are not part of sub_check_data
            "on_complete_script": sub_check_data.get("on_complete_script", []),
            "on_command_fail_script": sub_check_data.get("on_command_fail_script", []),
            "required_cwd": sub_check_data.get("required_cwd"),
            "fail_type": sub_check_data.get("fail_type"),
            "fail_criteria": sub_check_data.get("fail_criteria"),
        }
        temp_obj = Objective.from_dict(temp_obj_data)

        # If any sub-check fails, the entire checklist fails.
        if not checker_func(temp_obj, res):
            return False

    # If the loop completes, all sub-checks passed.
    return True


# --- Dispatch Map ---
# Maps an objective `type` from a YAML file to a checker function.

_CHECKER_MAP: Dict[str, Callable[[Objective, CommandResult], bool]] = {
    "command_run": _check_command_run,
    "path_exists": _check_path_exists,
    "path_not_exists": _check_path_not_exists,
    "file_contains": _check_file_contains,
    "any_command": _check_any_command,
    "cwd_is": _check_cwd_is,
    "manual_complete": _check_manual_complete,
    "checklist": _check_checklist,
}
