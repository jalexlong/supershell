import json
import os
from pathlib import Path
from typing import Dict
from .quest import Quest
from .agent import AgentGuide

def initialize_game():
    """Loads quests from JSON config and starts the AgentGuide."""
    
    # Resolve the path to the quests.json
    # Assumes config/ is one level up from supershell/
    base_dir = Path(__file__).resolve().parent.parent 
    config_path = base_dir / 'config' / 'quests.json'

    try:
        with open(config_path, 'r') as f:
            config_data = json.load(f)
    except FileNotFoundError:
        print(f"FATAL: Configuration file not found at: {config_path}")
        return
    except json.JSONDecodeError:
        print("FATAL: Error parsing quests.json. Check syntax.")
        return

    # Load all quests into a dictionary of Quest objects
    quest_instances: Dict[str, Quest] = {}
    for quest_data in config_data['quests']:
        try:
            quest = Quest(quest_data)
            quest_instances[quest.quest_id] = quest
        except ValueError as e:
            print(f"FATAL: Failed to load quest {quest_data.get('quest_id', 'Unknown')}: {e}")
            return
            
    # Start the Agent Guide with the loaded quests
    guide = AgentGuide(quests=quest_instances)
    guide.run_interaction_loop()

# --- DEMO ENVIRONMENT SETUP ---
def setup_demo_environment():
    """Sets up the initial state required for the L3Q1_SECURE_LOGS demo quest."""
    
    target_dir = Path("/home/student/game_world")
    insecure_file = target_dir / "access.log"

    # 1. Ensure the directory exists
    target_dir.mkdir(parents=True, exist_ok=True)
    
    # 2. Ensure the log file exists
    if not insecure_file.exists():
         with open(insecure_file, "w") as f:
             f.write("Insecure log file created by rogue process.\n")
    
    # 3. Set the initial insecure state (777 permissions, owned by student)
    # The '0o' prefix denotes octal for Python's os.chmod
    os.chmod(insecure_file, 0o777)
    
    # NOTE: Changing ownership to 'student' and back to 'root' requires
    # proper subprocess calls that often need root/sudo privileges.
    # For a simple demo, we rely on the student changing the owner to 'root'.
    # If the script is run by the student, it will be owned by them by default.
    
    print(f"DEMO SETUP COMPLETE: File '{insecure_file.name}' created with insecure permissions (777).")
    print("-------------------------------------------\n")


if __name__ == "__main__":
    setup_demo_environment()
    initialize_game()

