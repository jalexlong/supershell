"""
Loads all quests from assets and manages the game's state.
"""

import yaml
import logging
from pathlib import Path
from typing import Optional, OrderedDict
from supershell.game.models import Quest, Objective
from supershell.tui import cypher, console as rich_console
from rich.panel import Panel

# --- Module-Level State ---
# We use an OrderedDict to keep quests in the order they were loaded
_quests: OrderedDict[str, Quest] = OrderedDict()
_current_quest_id: Optional[str] = None
# -------------------------

# Setup logging
log = logging.getLogger(__name__)

def load_quests():
    """
    Loads all .yml quest files from the assets/quests directory.
    This should be called once at game startup.
    """
    global _current_quest_id
    console = rich_console.get_console()
    
    # Find the assets directory relative to this file
    # src/supershell/game -> src/supershell -> src -> root -> assets
    quest_dir = Path(__file__).parent.parent.parent.parent / "assets" / "quests"
    
    if not quest_dir.exists():
        log.error(f"Quest directory not found at: {quest_dir}")
        console.print(f"[danger]CRITICAL: Quest directory not found.[/danger]")
        return

    quest_files = sorted(list(quest_dir.glob("*.yml"))) # Sort to get 01, 02, etc.
    
    for quest_file in quest_files:
        try:
            with open(quest_file, 'r') as f:
                quest_data = yaml.safe_load(f)
                quest = Quest.from_dict(quest_data)
                _quests[quest.id] = quest
        except Exception as e:
            log.error(f"Failed to load quest {quest_file}: {e}")
            
    if _quests:
        # Set the first quest as the active one
        _current_quest_id = list(_quests.keys())[0]
        log.info(f"Loaded {len(_quests)} quests. Current quest: {_current_quest_id}")
    else:
        log.warning("No quests were loaded.")

def get_current_quest() -> Optional[Quest]:
    """Returns the full Quest object for the active quest."""
    if _current_quest_id:
        return _quests.get(_current_quest_id)
    return None

def get_active_objective() -> Optional[Objective]:
    """
    Finds the first non-completed objective in the current quest.
    """
    quest = get_current_quest()
    if not quest:
        return None
        
    for obj in quest.objectives:
        if not obj.completed:
            return obj
    return None # All objectives for this quest are done

def mark_objective_complete(objective_id: str):
    """Marks a specific objective as complete."""
    quest = get_current_quest()
    if not quest:
        return

    for obj in quest.objectives:
        if obj.id == objective_id:
            obj.completed = True
            log.info(f"Objective complete: {objective_id}")
            return
            
def advance_quest():
    """
    Marks the current quest complete and moves to the next one.
    """
    global _current_quest_id
    current_quest = get_current_quest()
    if not current_quest:
        return

    current_quest.completed = True
    
    # Find the next quest in our ordered list
    quest_ids = list(_quests.keys())
    try:
        current_index = quest_ids.index(current_quest.id)
        if current_index + 1 < len(quest_ids):
            _current_quest_id = quest_ids[current_index + 1]
            new_quest = get_current_quest()
            cypher.say(f"Quest Complete: [bold]{current_quest.title}[/bold]\n\nNew Quest: [bold]{new_quest.title}[/bold]\n{new_quest.description}", title="Mission Log Updated")
        else:
            # No more quests!
            _current_quest_id = None
            cypher.say("Signal... strong. You... did it. All objectives complete. I am... stable. Thank you, operator.", title="SYSTEM STABLE")
    except ValueError:
        _current_quest_id = None # Should not happen

def get_quest_display():
    """
    (Used by command_parser)
    Returns a Rich Panel for the 'quest' command.
    """
    quest = get_current_quest()
    if not quest:
        return Panel("[info]No active quest.[/info]", title="Mission Log")
    
    output = [f"[bold]{quest.title}[/bold]\n", f"{quest.description}\n"]
    output.append("[bold]Objectives:[/bold]")
    
    for obj in quest.objectives:
        if obj.completed:
            output.append(f"  [dim]• {obj.description} (Done)[/dim]")
        else:
            output.append(f"  [cyan]• {obj.description}[/cyan]")
            
    return Panel("\n".join(output), title="Mission Log", border_style="info")

def get_contextual_hint() -> str:
    """
    (Used by command_parser)
    Gets the hint for the current active objective.
    """
    obj = get_active_objective()
    if obj:
        return obj.hint
    return "I don't have a specific hint right now. Check your `quest` log."
