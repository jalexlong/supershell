"""
Data models for Quests and Objectives.
"""

from dataclasses import dataclass, field
from typing import Any

@dataclass
class Objective:
    """Represents a single task for the player."""
    id: str
    description: str
    type: str  # e.g., "command_run", "path_exists"
    criteria: dict[str, Any]
    hint: str
    success_message: str
    completed: bool = False

    @classmethod
    def from_dict(cls, data: dict):
        """Creates an Objective from a dictionary (from YAML)."""
        return cls(
            id=data['id'],
            description=data['description'],
            type=data['type'],
            criteria=data.get('criteria', {}),
            hint=data.get('hint', 'No hint available for this task.'),
            success_message=data.get('success_message', 'Objective complete.')
        )

@dataclass
class Quest:
    """Represents a full quest with multiple objectives."""
    id: str
    title: str
    description: str  # The "flavor text" for the quest
    objectives: list[Objective]
    completed: bool = False

    @classmethod
    def from_dict(cls, data: dict):
        """Creates a Quest from a dictionary (from YAML)."""
        objectives = [
            Objective.from_dict(obj_data) 
            for obj_data in data.get('objectives', [])
        ]
        return cls(
            id=data['id'],
            title=data['title'],
            description=data['description'],
            objectives=objectives
        )
