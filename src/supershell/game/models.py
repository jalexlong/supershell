"""
Data models for Quests and Objectives.
"""

from dataclasses import dataclass
from typing import Any


@dataclass
class Objective:
    """Represents a single task for the player."""

    id: str
    description: str
    type: str  # e.g., "command_run", "path_exists"
    criteria: dict[str, Any]
    hint: str
    completed: bool = False

    @classmethod
    def from_dict(cls, data: dict):
        """Creates an Objective from a dictionary."""
        return cls(
            id=data["id"],
            description=data["description"],
            type=data["type"],
            criteria=data.get("criteria", {}),
            hint=data.get("hint", "No hint available for this task."),
        )
