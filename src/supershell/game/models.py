"""
Data models for Quests and Objectives, loaded from YAML.
Also includes the CommandResult dataclass for passing command information.
"""

import logging
import os
import shutil
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Union

log = logging.getLogger(__name__)


@dataclass
class CommandResult:
    """
    Represents the result of an executed shell command.
    Used to pass command details from the client to the daemon for validation.
    """

    command: str
    return_code: int
    # stdout and stderr are intentionally omitted as the daemon
    # cannot inspect them in the new architecture.


@dataclass
class Objective:
    """Represents a single task (a "Step") for the player."""

    id: str
    type: str
    criteria: Union[
        Dict[str, Any], List[Dict[str, Any]]
    ]  # Modified to accept Dict OR List[Dict]
    hint: str
    on_complete_script: List[Dict[str, Any]] = field(default_factory=list)
    on_command_fail_script: List[Dict[str, Any]] = field(default_factory=list)
    description: str = ""
    completed: bool = False
    required_cwd: Optional[str] = None  # Optional required current working directory
    fail_type: Optional[str] = None  # New: Type of check for failure condition
    fail_criteria: Optional[Union[Dict[str, Any], List[Dict[str, Any]]]] = (
        None  # New: Criteria for failure condition
    )

    @classmethod
    def from_dict(cls, data: dict):
        """Creates an Objective from a dictionary (from YAML)."""
        criteria_data = data.get("criteria", {})
        # Check if criteria is a list (for multi_path_exists)
        if isinstance(criteria_data, list):
            criteria = criteria_data
        else:
            criteria = criteria_data  # Keep original logic if it's a single dict

        return cls(
            id=data.get("id", "MISSING_ID"),
            type=data.get("type", "any_command"),
            criteria=criteria,
            hint=data.get("hint", "No hint available for this task."),
            on_complete_script=data.get("on_complete_script", []),
            on_command_fail_script=data.get("on_command_fail_script", []),
            description=data.get("description", ""),
            required_cwd=data.get("required_cwd"),  # New: Load the required_cwd
            # New failure fields
            fail_type=data.get("fail_type"),  # Directly get from data
            fail_criteria=data.get("fail_criteria"),  # Directly get from data
        )


@dataclass
class Quest:
    """Represents a full quest, loaded from YAML."""

    id: str
    title: str
    description: str
    on_quest_start: List[Dict[str, Any]]
    objectives: list[Objective]
    on_load_sync: List[Dict[str, Any]]
    on_hard_fail_script: List[Dict[str, Any]] = field(default_factory=list)

    _tracked_files: set = field(default_factory=set)
    _tracked_dirs: set = field(default_factory=set)

    @classmethod
    def from_yaml(cls, data: dict):
        """Creates a full Quest object from YAML data."""
        return cls(
            id=data.get("id", "MISSING_ID"),
            title=data.get("title", "Untitled Quest"),
            description=data.get("description", ""),
            on_quest_start=data.get("on_quest_start", []),
            objectives=[Objective.from_dict(obj) for obj in data.get("objectives", [])],
            on_load_sync=data.get("on_load_sync", []),
            on_hard_fail_script=data.get("on_hard_fail_script", []),
        )

    # --- File Tracking & Cleanup Logic ---

    def _spawn_file(self, path: str, content: str = ""):
        """Creates a file AND tracks it for cleanup."""
        full_path = os.path.expanduser(path)
        try:
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, "w") as f:
                f.write(content)
            log.info(f"Spawned file: {full_path}")
            self._tracked_files.add(full_path)
        except (IOError, OSError) as e:
            log.error(f"Could not create file {full_path}: {e}")

    def _spawn_dir(self, path: str):
        """Creates a directory AND tracks it for cleanup."""
        full_path = os.path.expanduser(path)
        try:
            os.makedirs(full_path, exist_ok=True)
            log.info(f"Spawned tracked directory: {full_path}")
            self._tracked_dirs.add(full_path)
        except (IOError, OSError) as e:
            log.error(f"Could not create directory {full_path}: {e}")

    def _cleanup_quest_files(self):
        """Removes all files and dirs created by this quest."""
        log.info(f"Cleaning up files for quest: {self.id}")
        for f_path in self._tracked_files:
            try:
                os.remove(f_path)
            except FileNotFoundError:
                pass

        for d_path in self._tracked_dirs:
            try:
                shutil.rmtree(d_path)
            except FileNotFoundError:
                pass

        self._tracked_files.clear()
        self._tracked_dirs.clear()

    def get_objective_by_id(self, objective_id: str) -> Optional[Objective]:
        """Retrieves an objective from this quest by its ID."""
        for obj in self.objectives:
            if obj.id == objective_id:
                return obj
        return None

    # --- Event Handling (Called by Daemon) ---
    def handle_event(self, objective_id: str, command_result: CommandResult):
        """
        Called by the daemon when an objective is completed or failed.
        Triggers on_complete_script or on_command_fail_script.
        """
        from supershell.game import actions

        obj = self.get_objective_by_id(objective_id)
        if not obj:
            log.warning(
                f"Attempted to handle event for non-existent objective: {objective_id}"
            )
            return

        if obj.completed:
            log.info(f"Running on_complete_script for objective: {objective_id}")
            for action_data in obj.on_complete_script:
                run_params = action_data.copy()
                run_params["command_result"] = command_result
                actions.run_action(run_params)
        else:  # This path implies a 'FAIL' status from the validator
            log.info(f"Running on_command_fail_script for objective: {objective_id}")
            for action_data in obj.on_command_fail_script:
                run_params = action_data.copy()
                run_params["command_result"] = command_result
                actions.run_action(run_params)
