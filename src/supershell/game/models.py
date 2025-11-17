from dataclasses import dataclass, field
from typing import Any, Dict, List, Union


@dataclass
class Objective:
    id: str
    description: str
    type: str
    criteria: Union[Dict[str, Any], List[Dict[str, Any]]]
    hint: str
    completed: bool = False
    fail_type: str | None = None
    fail_criteria: dict[str, Any] = field(default_factory=dict)
    required_cwd: str | None = None

    @classmethod
    def from_dict(cls, data: dict):
        return cls(
            id=data["id"],
            description=data["description"],
            type=data["type"],
            criteria=data.get("criteria", {}),
            hint=data.get("hint", "No hint available for this task."),
        )
