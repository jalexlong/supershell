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

    def __post_init__(self):
        valid_types = {
            "command_run",
            "path_exists",
            "file_contains",
            "any_command",
            "cwd_is",
            "path_not_exists",
            "manual_complete",
            "checklist",
        }
        if self.type not in valid_types:
            raise ValueError(
                f"Invalid objective type: {self.type}. Must be one of {valid_types}"
            )
        if self.fail_type and self.fail_type not in valid_types:
            raise ValueError(
                f"Invalid objective fail_type: {self.fail_type}. Must be one of {valid_types}"
            )

    @classmethod
    def from_dict(cls, data: dict):
        return cls(
            id=data["id"],
            description=data["description"],
            type=data["type"],
            criteria=data.get("criteria", {}),
            hint=data.get("hint", "No hint available for this task."),
        )
