"""
Data models for Quests and Objectives.
"""

import logging
import os
from dataclasses import dataclass, field
from typing import Any, Dict, List

logger = logging.getLogger(__name__)


@dataclass
class Objective:
    """Represents a single task (a "Step") for the player."""

    id: str
    type: str
    criteria: dict[str, Any]
    hint: str
    # This is the "script" of actions to run on completion
    on_complete_script: List[Dict[str, Any]] = field(default_factory=list)

    # This is what the quest log will show
    description: str = ""

    # This is managed by the quest_manager
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
            description=data.get("description", ""),
        )


@dataclass
class Quest:
    """Represents a full quest, loaded from YAML."""

    id: str
    title: str
    description: str
    on_quest_start: List[Dict[str, Any]]  # The intro "cutscene" script
    objectives: list[Objective]

    # These are for the file cleanup system
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
        )

    def _cleanup_quest_files(self):
        """Removes all files and dirs created by this quest."""
        log = logging.getLogger(__name__)
        log.info(f"Cleaning up files for quest: {self.id}")

        # We need to import shutil here, inside the method
        import shutil

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
