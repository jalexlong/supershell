import logging
import os
import socket

from supershell.game import event_handler, objective_checker, quest_manager
from supershell.shell import executor, interpreter, parser
from supershell.tui import dialogue
from supershell.tui.console import get_console

logging.basicConfig(
    level=logging.INFO,
    filename="supershell.log",
    filemode="w",
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
log = logging.getLogger(__name__)


def main_loop():
    console = get_console()

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
        user = "anomaly"
    try:
        host = socket.gethostname().split(".")[0]
    except Exception:
        host = "localhost"
    user_host_str = f"{user}@{host}"

    event_handler.handle_game_start(user, host)

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

        status, objective_id = objective_checker.check(result)

        if status == "SUCCESS":
            if objective_id is not None:
                event_handler.handle_objective_completion(objective_id)
            else:
                log.error("Objective check returned SUCCESS but objective_id was None.")

        elif status == "FAIL":
            active_quest = quest_manager.get_current_quest()
            if active_quest:
                active_quest.on_objective_failure(result)

    quest_manager.cleanup_all_quest_files()
    dialogue.say("Goodbye...", character="system")
