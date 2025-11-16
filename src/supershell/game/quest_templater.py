"""
A utility script to generate YAML templates for quests and objectives.

Run this script to get examples of quest and objective structures
that can be copied and adapted for new quests in assets/quests/*.yml.
"""

from typing import Any, Dict, List, Optional, Union

import yaml

# --- Templates for Objective Criteria ---
# These structures are based on how quest_validator.py expects criteria to be.
OBJECTIVE_CRITERIA_TEMPLATES: Dict[str, Union[Dict[str, Any], List[Dict[str, Any]]]] = {
    "command_run": {"command": "expected_command_name"},
    "path_exists": {
        "path": "~/some/directory/or/file",
        "type": "file",  # or "dir"
    },
    "file_contains": {
        "path": "~/some/file.txt",
        "content": "expected_text_inside_file",
        # "content_from_save": "saved_data_key" # Uncomment to use saved data
    },
    "any_command": {
        # No specific criteria needed, any command will satisfy if it runs without error
    },
    "cwd_is": {"path": "~/expected/current/directory"},
    "path_not_exists": {"path": "~/path/that/should/not/exist"},
    "manual_complete": {
        # This objective type is completed by game actions, not player commands
    },
    "multi_path_exists": [
        {
            "path": "~/path/to/file1",
            "type": "file",  # or "dir"
            "exist": True,  # True if path should exist, False if it should not
        },
        {"path": "~/path/to/dir2", "type": "dir", "exist": False},
        # Add more items as needed
    ],
}

# --- Templates for Actions ---
# These structures are based on the functions in actions.py and their expected arguments.
ACTION_TEMPLATES: Dict[str, Dict[str, Any]] = {
    "say": {
        "action": "say",
        "character": "cypher",  # or glitch, hunter, system, quest, mission
        "message": "This is a dialogue message from the character.",
    },
    "advance_objective": {"action": "advance_objective"},
    "advance_quest": {"action": "advance_quest"},
    "track_dir": {"action": "track_dir", "path": "~/path/to/directory/to/track"},
    "track_file": {"action": "track_file", "path": "~/path/to/file/to/track"},
    "untrack_file": {"action": "untrack_file", "path": "~/path/to/file/to/untrack"},
    "spawn_tracked_file": {
        "action": "spawn_tracked_file",
        "path": "~/path/to/new_file.txt",
        "content": "This is the content of the new file.",
    },
    "spawn_tracked_dir": {
        "action": "spawn_tracked_dir",
        "path": "~/path/to/new_directory",
    },
    "conditional_say_on_fail": {
        "action": "conditional_say_on_fail",
        "character": "glitch",
        "message": "It seems your command failed! Try something else.",
        "if_command": "rm",  # Optional: only trigger if 'rm' command was used
        "if_args_contain": "-rf",  # Optional: only trigger if args contain '-rf'
    },
    "reset_current_quest": {"action": "reset_current_quest"},
    "conditional_hard_fail": {
        "action": "conditional_hard_fail",
        "if_command": "rm",  # Optional: only trigger if 'rm' command was used
        "if_args_contain": "-rf",  # Optional: only trigger if args contain '-rf'
        "if_return_code": 1,  # Optional: only trigger if command failed with specific return code
    },
    "generate_and_save_password": {"action": "generate_and_save_password"},
    "cleanup_all_tracked_files": {"action": "cleanup_all_tracked_files"},
}


def create_objective_template(
    objective_id: str,
    objective_type: str,
    description: str = "A description of what the player needs to do.",
    hint: str = "A helpful hint for the player if they get stuck.",
    required_cwd: Optional[str] = None,
) -> Dict[str, Any]:
    """Generates a template for a single objective."""
    criteria = OBJECTIVE_CRITERIA_TEMPLATES.get(objective_type)
    if criteria is None:
        raise ValueError(f"Unknown objective type: {objective_type}")

    template = {
        "id": objective_id,
        "type": objective_type,
        "description": description,
        "hint": hint,
        "criteria": criteria,
        "on_complete_script": [],
        "on_command_fail_script": [],
    }
    if required_cwd:
        template["required_cwd"] = required_cwd
    return template


def create_action_template(action_name: str) -> Dict[str, Any]:
    """Generates a template for a single action."""
    template = ACTION_TEMPLATES.get(action_name)
    if template is None:
        raise ValueError(f"Unknown action: {action_name}")
    return template


def create_quest_template(
    quest_id: str, title: str, description: str
) -> Dict[str, Any]:
    """Generates a template for a complete quest with placeholder objectives and actions."""

    # Example objective for the quest
    example_objective = create_objective_template(
        objective_id=f"{quest_id}_obj_1",
        objective_type="any_command",
        description="Run any command to start the quest.",
        hint="Just type 'ls' or 'pwd' to get started!",
    )
    example_objective["on_complete_script"].append(
        create_action_template("advance_objective")
    )
    example_objective["on_command_fail_script"].append(
        create_action_template("conditional_say_on_fail")
    )

    return {
        "id": quest_id,
        "title": title,
        "description": description,
        "on_quest_start": [
            create_action_template("say"),
            {
                "action": "say",
                "character": "cypher",
                "message": f"Welcome to the quest: [bold]{title}[/bold]!",
            },
            {"action": "advance_objective"},
        ],
        "objectives": [
            example_objective,
            create_objective_template(
                objective_id=f"{quest_id}_obj_2",
                objective_type="path_exists",
                description="Create a file named 'report.txt' in your home directory.",
                hint="Use 'touch' to create a file.",
                required_cwd="~",
            ),
            create_objective_template(
                objective_id=f"{quest_id}_obj_3",
                objective_type="file_contains",
                description="Add the text 'mission critical' to 'report.txt'.",
                hint="You might need to use 'echo' and redirection (>) or (>>).",
                required_cwd="~",
            ),
            # Add more objectives as needed
        ],
        "on_load_sync": [
            # Actions to set up the world state when the quest is loaded or reset
            {"action": "spawn_tracked_dir", "path": "~/data"},
            {
                "action": "spawn_tracked_file",
                "path": "~/data/secret.md",
                "content": "The password is {{secret_password}}.",
            },
            {
                "action": "generate_and_save_password"
            },  # Important if you use {{secret_password}}
        ],
        "on_hard_fail_script": [
            {
                "action": "say",
                "character": "hunter",
                "message": "You made a critical error. Restarting quest.",
            },
            {"action": "reset_current_quest"},
        ],
    }


def main():
    print("--- Quest Template Generator ---")
    print("This script helps you create YAML templates for your quests.")
    print("Copy the output below into a .yml file in your assets/quests/ directory.\n")

    print("\n--- Example Full Quest Template ---")
    full_quest = create_quest_template(
        quest_id="new_quest_001",
        title="My First Custom Quest",
        description="This is an example quest to guide you through the process of creating a new adventure.",
    )
    print(yaml.dump(full_quest, indent=2, sort_keys=False))

    print("\n--- Example Objective Templates (by type) ---")
    for obj_type in OBJECTIVE_CRITERIA_TEMPLATES:
        print(f"\n# Objective Type: {obj_type}")
        obj_template = create_objective_template(
            objective_id=f"example_{obj_type}",
            objective_type=obj_type,
            description=f"Complete this task using a '{obj_type}' objective.",
            hint=f"Hint for '{obj_type}' objective.",
        )
        print(yaml.dump(obj_template, indent=2, sort_keys=False))

    print("\n--- Example Action Templates (by name) ---")
    for action_name in ACTION_TEMPLATES:
        print(f"\n# Action: {action_name}")
        action_template = create_action_template(action_name)
        print(yaml.dump(action_template, indent=2, sort_keys=False))

    print("\n--- End of Templates ---")
    print(
        "Remember to adjust IDs, descriptions, criteria, and scripts to your quest's needs."
    )


if __name__ == "__main__":
    main()
