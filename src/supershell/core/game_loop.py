import logging
import os
import socket

from supershell.game import actions, event_handler, quest_manager, quest_validator
from supershell.shell import executor, interpreter, parser
from supershell.tui.console import get_console

# --- Custom Logging Setup ---
# Get the root logger
root_logger = logging.getLogger()
root_logger.setLevel(
    logging.DEBUG
)  # Set the root logger to DEBUG to capture all levels

# Create formatters
formatter = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")

# Handler for INFO and above to main log file
file_handler_info = logging.FileHandler("supershell.log", mode="w")
file_handler_info.setLevel(logging.INFO)
file_handler_info.setFormatter(formatter)
root_logger.addHandler(file_handler_info)

# Handler for DEBUG and above to a separate debug log file
file_handler_debug = logging.FileHandler("supershell_debug.log", mode="w")
file_handler_debug.setLevel(logging.DEBUG)
file_handler_debug.setFormatter(formatter)
root_logger.addHandler(file_handler_debug)

# Handler for INFO and above to console (optional, for real-time feedback)
# console_handler = logging.StreamHandler()
# console_handler.setLevel(logging.INFO)
# console_handler.setFormatter(formatter)
# root_logger.addHandler(console_handler)

# --- End Custom Logging Setup ---

log = logging.getLogger(__name__)


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
    log.debug("Game starting...")
    event_handler.handle_game_start(user, host)
    log.debug("Game started. Entering main loop.")

    # --- 3. THE MAINLOOP ---
    while True:
        log.debug("--- Main Loop Iteration Start ---")
        cwd = os.getcwd()
        cwd = cwd.replace(home_dir, "~", 1) if cwd.startswith(home_dir) else cwd
        prompt_parts = [
            ("class:userhost", user_host_str),
            ("", ":"),
            ("class:cwd", f"{cwd}"),
            ("", "$ "),
        ]

        command_str = interpreter.get_command(prompt_parts)
        log.debug(f"User entered command: '{command_str}'")

        if command_str == "exit":
            log.info("User requested exit. Breaking main loop.")
            break
        if not command_str:
            log.debug("Empty command entered. Continuing loop.")
            continue

        log.debug(f"Parsing command: '{command_str}'")
        if parser.parse_and_handle(command_str):
            log.debug(f"Command '{command_str}' handled by parser. Skipping executor.")
            continue

        log.debug(f"Executing command: '{command_str}'")
        result = executor.execute_command(command_str)
        log.debug(f"Command execution complete. Result: {result}")

        if result.stdout:
            log.debug(f"Command stdout: {result.stdout.strip()}")
            console.print(f"[stdout]{result.stdout}[/stdout]")
        if result.stderr:
            log.debug(f"Command stderr: {result.stderr.strip()}")
            console.print(f"[stderr]{result.stderr}[/stderr]")

        # 4. CHECK & HANDLE EVENTS
        log.debug("Checking for objective completion...")
        completed_id = quest_validator.check(command_str, result)
        log.debug(
            f"Objective Check returned: {completed_id if completed_id else 'None'}"
        )

        if completed_id:
            log.info(f"Objective '{completed_id}' completed! Handling completion.")
            event_handler.handle_objective_completion(completed_id)
            log.debug(f"Handled objective completion for '{completed_id}'.")
        else:
            # Handle command failure (soft fail)
            log.debug("No objective completed.")
            active_objective = quest_manager.get_active_objective()
            if active_objective and active_objective.on_command_fail_script:
                # Pass the command_result to actions for contextual feedback
                log.debug(
                    f"Command failed for active objective '{active_objective.id}'. Running on_command_fail_script"
                )
                for action_data in active_objective.on_command_fail_script:
                    # If action expects 'command_result' it will be handled by **kwargs
                    run_params = action_data.copy()
                    run_params["command_result"] = result  # Add the result to params
                    log.debug(
                        f"Running fail action: {run_params.get('action')} with params {run_params}"
                    )
                    actions.run_action(run_params)
            else:
                log.debug("No active objective or no 'on_command_fail_script' to run.")

        log.debug("--- Main Loop Iteration End ---")

    quest_manager.cleanup_all_quest_files()
    log.debug("Cleanup all quest files on exit.")
    log.info("Game loop finished.")
