import os
import logging
import socket

from supershell.tui.console import get_console
from supershell.shell import interpreter, executor, parser
from supershell.tui import dialogue
from supershell.game import event_handler, objective_checker, quest_manager

# --- Setup basic logging ---
logging.basicConfig(level=logging.INFO, filename='supershell.log', filemode='w',
                    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
# ---

def main_loop():
    """
    This is the simple, synchronous (blocking) main game loop.
    """
    console = get_console()
    
    # --- 1. SETUP ---
    try:
        quest_manager.load_quests()
    except Exception as e:
        console.print(f"[danger]Failed to load quests: {e}[/danger]")
        logging.critical(f"Failed to load quests: {e}", exc_info=True)
        return

    home_dir = os.path.expanduser("~")
    os.chdir(home_dir)

    try: user = os.getlogin()
    except OSError: user = "anomaly"
    try: host = socket.gethostname()
    except Exception: host = "localhost"
    user_host_str = f"{user}@{host}"
    
    # --- 2. START THE GAME ---
    first_quest = quest_manager.get_current_quest()
    if first_quest:
        # This will print Cypher's "Whoa... a new process?"
        # text from the quest description.
        dialogue.say(f"[bold]{first_quest.title}[/bold]\n\n{first_quest.description}", actor="cypher")
    else:
        # Fallback if no quests are found
        dialogue.say("No quests loaded. System idle.", actor="cypher")
        event_handler.handle_game_start(user, host)

    
    # --- 3. THE MAINLOOP ---
    while True:
        cwd = os.getcwd(); cwd = cwd.replace(home_dir, "~", 1) if cwd.startswith(home_dir) else cwd
        prompt_parts = [
            ('class:userhost', user_host_str), ('', ":"),
            ('class:cwd', f"{cwd}"), ('', "$ "),
        ]
        
        command_str = interpreter.get_command(prompt_parts)
        
        if command_str == "exit": break
        if not command_str: continue

        if parser.parse_and_handle(command_str): continue
        
        result = executor.execute_command(command_str)
        
        if result.stdout: console.print(f"[stdout]{result.stdout}[/stdout]")
        if result.stderr: console.print(f"[stderr]{result.stderr}[/stderr]")
            
        # 4. CHECK & HANDLE EVENTS
        completed_id = objective_checker.check(result)
        if completed_id:
            event_handler.handle_objective_completion(completed_id)

    quest_manager.cleanup_all_quest_files()
    dialogue.say("Goodbye", actor="cypher")

