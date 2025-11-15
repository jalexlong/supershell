"""
Data models for Quests and Objectives, loaded from YAML.
"""

import logging
import os
import shutil
from dataclasses import dataclass, field
from typing import Any, Dict, List

log = logging.getLogger(__name__)


@dataclass
class Objective:
    """Represents a single task (a "Step") for the player."""

    id: str
    type: str
    criteria: dict[str, Any]
    hint: str
    on_complete_script: List[Dict[str, Any]] = field(default_factory=list)
    on_command_fail_script: List[Dict[str, Any]] = field(default_factory=list)
    description: str = ""
    completed: bool = False

    @classmethod
    def from_dict(cls, data: dict):
        """Creates an Objective from a dictionary (from YAML)."""
        return cls(
            id=data.get("id", "MISSING_ID"),
            type=data.get("type", "any_command"),
            criteria=data.get("criteria", {}),
            hint=data.get("hint", "No hint available for this task."),
            on_complete_script=data.get("on_complete_script", []),
            on_command_fail_script=data.get("on_command_fail_script", []),
            description=data.get("description", ""),
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
