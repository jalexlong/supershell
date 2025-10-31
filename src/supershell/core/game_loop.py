import os
import logging
import socket

from supershell.tui.console import get_console
from supershell.shell import interpreter, executor, parser
from supershell.tui import cypher
from supershell.game import quest_manager, objective_checker

# --- Setup basic logging ---
# You can get much fancier, but this is a start
logging.basicConfig(level=logging.INFO, filename='supershell.log', filemode='w',
                    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
# ---

def main_loop():
    console = get_console()
    
    # --- LOAD QUESTS AT STARTUP ---
    try:
        quest_manager.load_quests()
    except Exception as e:
        console.print(f"[danger]Failed to load quests: {e}[/danger]")
        logging.critical(f"Failed to load quests: {e}", exc_info=True)
        return # Can't run the game
        
    # ---
    
    cypher.say("System online. Welcome to supershell, operator. Type `help` if you're lost.")
    
    # --- Print first quest info ---
    first_quest = quest_manager.get_current_quest()
    if first_quest:
        cypher.say(f"[bold]{first_quest.title}[/bold]\n\n{first_quest.description}", title="New Mission")
    # ---

    home_dir = os.path.expanduser("~")

    # Get user and host info
    try:
        user = os.getlogin()
    except OSError:
        user = "operator"  # Fallback for environments where getlogin fails

    try:
        host = socket.gethostname()
    except Exception:
        host = "supershell"  # Fallback

    user_host_str = f"{user}@{host}"
    
    while True:
        # 1. READ
        cwd = os.getcwd()
        if cwd.startswith(home_dir):
            # Replace the home path with '~' for a cleaner look
            cwd = cwd.replace(home_dir, "~", 1)

        # prompt_toolkit expects a list of (style, text) tuples
        prompt = [
                ('class:userhost', user_host_str),
                ('class:cwd', f" {cwd}"),  # Note the leading space
                ('', " $ "),               # Empty class = default terminal color (white)
        ]

        command_str = interpreter.get_command(prompt)

        # 2. EVAL
        if not command_str:
            continue
        
        if command_str == "exit":
            break

        if parser.parse_and_handle(command_str):
            continue
        
        result = executor.execute_command(command_str)

        # 3. PRINT
        if result.stdout:
            console.print(f"[stdout]{result.stdout}[/stdout]")
        if result.stderr:
            console.print(f"[stderr]{result.stderr}[/stderr]")

        # 4. CHECK
        objective_checker.check(result)
        
    cypher.say("...Signal lost. Disconnecting...")
