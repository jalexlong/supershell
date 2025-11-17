import logging
import os
import socket

from supershell.game import actions, event_handler, quest_manager, quest_validator
from supershell.shell import executor, interpreter, parser
from supershell.tui.console import get_console

# --- Setup basic logging ---
logging.basicConfig(
    level=logging.INFO,
    filename="supershell.log",
    filemode="w",
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
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

    try:
        user = os.getlogin()
    except OSError:
        user = "operator"
    try:
        host = socket.gethostname().split(".")[0]
    except Exception:
        host = "localhost"
    user_host_str = f"{user}@{host}"

    # --- 2. START THE GAME ---
    event_handler.handle_game_start(user, host)

    # --- 3. THE MAINLOOP ---
    while True:
        cwd = os.getcwd()
        cwd = cwd.replace(home_dir, "~", 1) if cwd.startswith(home_dir) else cwd
        prompt_parts = [
            ("class:userhost", user_host_str),
            ("", ":"),
            ("class:cwd", f"{cwd}"),
            ("", "$ "),
        ]

        command_str = interpreter.get_command(prompt_parts)

        if command_str == "exit":
            break
        if not command_str:
            continue

        if parser.parse_and_handle(command_str):
            continue

        result = executor.execute_command(command_str)

        if result.stdout:
            console.print(f"[stdout]{result.stdout}[/stdout]")
        if result.stderr:
            console.print(f"[stderr]{result.stderr}[/stderr]")

        # 4. CHECK & HANDLE EVENTS
        completed_id = quest_validator.check(command_str, result)
        # log.info(f"Objective Check returned: {completed_id}")

        if completed_id:
            event_handler.handle_objective_completion(completed_id)
        else:
            # New: Handle command failure (soft fail)
            active_objective = quest_manager.get_active_objective()
            if active_objective and active_objective.on_command_fail_script:
                # Pass the command_result to actions for contextual feedback
                for action_data in active_objective.on_command_fail_script:
                    # If action expects 'command_result' it will be handled by **kwargs
                    run_params = action_data.copy()
                    run_params["command_result"] = result  # Add the result to params
                    actions.run_action(run_params)

    quest_manager.cleanup_all_quest_files()
