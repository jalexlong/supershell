"""
Checks the result of a command against active quest objectives.
This version uses a "dispatch map" (dictionary) and returns
a "tri-state" status (SUCCESS, FAIL, CONTINUE).
"""

import logging
import os
from typing import Any, Callable, Dict, Optional, Tuple

from supershell.game import quest_manager
from supershell.shell.executor import CommandResult

log = logging.getLogger(__name__)

# --- Type definition for our checker functions ---
CheckerFunc = Callable[[Dict[str, Any], CommandResult], bool]


# --- 1. ALL CHECKER FUNCTIONS ARE DEFINED FIRST ---


def _check_command_run(criteria: Dict[str, Any], res: CommandResult) -> bool:
    if res.return_code != 0:
        return False
    command_ran = res.command.strip().split()[0].lower()
    expected_command = criteria.get("command")
    return command_ran == expected_command


def _check_path_exists(criteria: Dict[str, Any], res: CommandResult) -> bool:
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


def _check_any_command(criteria: Dict[str, Any], res: CommandResult) -> bool:
    return res.return_code == 0


def _check_cwd_is(criteria: Dict[str, Any], res: CommandResult) -> bool:
    path_to_check = criteria.get("path")
    if not path_to_check:
        return False
    return os.getcwd() == os.path.expanduser(path_to_check)


def _check_path_not_exists(criteria: Dict[str, Any], res: CommandResult) -> bool:
    path_to_check = criteria.get("path")
    if not path_to_check:
        return False
    return not os.path.exists(os.path.expanduser(path_to_check))


def _check_file_contains(criteria: Dict[str, Any], res: CommandResult) -> bool:
    path_to_check = os.path.expanduser(str(criteria.get("path")))
    content_to_find = criteria.get("content")
    if not path_to_check or not content_to_find:
        return False

    if os.path.isfile(path_to_check):
        try:
            with open(path_to_check, "r") as f:
                return content_to_find in f.read()
        except (IOError, OSError):
            return False

    elif os.path.isdir(path_to_check):
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


def _check_manual_complete(criteria: Dict[str, Any], res: CommandResult) -> bool:
    return False


def _check_checklist(criteria: Dict[str, Any], res: CommandResult) -> bool:
    """
    Checks if a "checklist" of sub-objectives are all true
    by re-using the main OBJECTIVE_CHECKERS map.
    """
    sub_objectives_data = criteria.get("objectives", [])
    if not sub_objectives_data:
        log.warning("Checklist objective has no 'objectives' list.")
        return False

    for sub_data in sub_objectives_data:
        sub_type = sub_data.get("type", "")
        sub_criteria = sub_data.get("criteria", {})

        # --- Check each sub-objective for completion ---
        checker_function = OBJECTIVE_CHECKERS.get(sub_type)

        if checker_function:
            if not checker_function(sub_criteria, res):
                return False  # One check failed
        else:
            log.warning(f"Checklist: Unknown sub-objective type: {sub_type}")
            return False  # A sub-objective is invalid

    return True  # All checks passed


# --- 2. THIS IS THE DISPATCH MAP ---
OBJECTIVE_CHECKERS: Dict[str, CheckerFunc] = {
    "command_run": _check_command_run,
    "path_exists": _check_path_exists,
    "file_contains": _check_file_contains,
    "any_command": _check_any_command,
    "cwd_is": _check_cwd_is,
    "path_not_exists": _check_path_not_exists,
    "manual_complete": _check_manual_complete,
    "checklist": _check_checklist,
}


def _run_check(obj_type: str, criteria: Dict[str, Any], res: CommandResult) -> bool:
    """Helper to run a check using the dispatch map."""
    checker_function = OBJECTIVE_CHECKERS.get(obj_type)
    if checker_function:
        return checker_function(criteria, res)
    else:
        log.warning(f"No checker found for objective type: {obj_type}")
        return False


# --- 3. THE TRI-STATE 'check' FUNCTION ---
def check(command_result: CommandResult) -> Tuple[str, Optional[str]]:
    """
    Checks the command result against the active objective.
    Returns a (STATUS, objective_id) tuple.
    STATUS can be "SUCCESS", "FAIL", or "CONTINUE".
    """
    active_obj = quest_manager.get_active_objective()
    if not active_obj:
        return "CONTINUE", None

    log.debug(
        f"Checking {command_result.command} against {active_obj.id} ({active_obj.type})"
    )

    try:
        # 0. NEW: Check if we are in the required CWD
        if active_obj.required_cwd:
            full_required_path = os.path.expanduser(active_obj.required_cwd)
            if os.getcwd() != full_required_path:
                # We are not in the right directory, so we can't succeed.
                # Don't check for success.

                # Check for failure
                if active_obj.fail_type and _run_check(
                    active_obj.fail_type, active_obj.fail_criteria, command_result
                ):
                    log.info(f"Objective FAILED (Wrong CWD): {active_obj.id}")
                    return "FAIL", active_obj.id

                # Otherwise, just continue
                return "CONTINUE", None

        # 1. Check for SUCCESS
        if _run_check(active_obj.type, active_obj.criteria, command_result):
            log.info(f"Objective complete: {active_obj.id}")
            quest_manager.mark_objective_complete(active_obj.id)
            return "SUCCESS", active_obj.id

        # 2. Check for FAIL
        if active_
