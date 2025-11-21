"""
The Supershell Daemon.
Runs in the background, maintains game state, and processes requests.
"""

import asyncio
import json
import logging
import os
import signal
import sys
from typing import Any, Dict, List, TypedDict  # Added Dict for request typing

from supershell.game import quest_manager, quest_validator
from supershell.game.models import CommandResult  # Now imported from game.models
from supershell.tui import dialogue

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
log = logging.getLogger("daemon")


# Define the expected structure of the response dictionary
class DaemonResponse(TypedDict):
    exit_code: int
    output: List[str]


class GameServer:
    def __init__(self):
        self.output_buffer: List[str] = []  # Explicitly typed as List[str]

    def capture_output(self, text: str):
        """Hook for dialogue.say to capture text instead of printing."""
        self.output_buffer.append(text)

    async def handle_client(self, reader, writer):
        # 1. Read Request
        data = await reader.read()
        try:
            request: Dict[str, Any] = json.loads(
                data.decode("utf-8")
            )  # Explicitly type request
        except json.JSONDecodeError:
            log.error("Failed to decode JSON request.", exc_info=True)
            writer.close()
            return
        except Exception as e:
            log.error(f"Unexpected error reading client data: {e}", exc_info=True)
            writer.close()
            return

        # 2. Setup Context
        # Clear buffer for this new request
        self.output_buffer = []
        # Point dialogue to our buffer
        dialogue.set_output_handler(self.capture_output)

        # Sync Daemon CWD with Client CWD
        # This ensures os.path.exists() checks the right place
        client_cwd = request.get("cwd")
        if client_cwd and os.path.exists(client_cwd):
            os.chdir(client_cwd)

        # Initialize response with default values
        response: DaemonResponse = {"exit_code": 0, "output": []}
        hook_type = request.get("type")
        args = request.get("args", [])

        log.info(f"Processing {hook_type} in {client_cwd} with args: {args}")

        # 3. Process Logic
        try:
            if hook_type == "init":
                self.handle_init()
            elif hook_type == "pre_exec":
                response["exit_code"] = self.handle_pre_exec(args)
            elif hook_type == "post_exec":
                self.handle_post_exec(args)
            elif hook_type == "game_cmd":
                self.handle_game_cmd(args)
        except Exception as e:
            log.error(
                f"Error processing request for hook_type {hook_type}: {e}",
                exc_info=True,
            )
            self.output_buffer.append(f"[bold red]System Error:[/bold red] {e}")
            response["exit_code"] = 1  # Indicate an error

        # 4. Set output and Send Response
        response["output"] = self.output_buffer  # Assign output first
        writer.write(json.dumps(response).encode("utf-8"))
        await writer.drain()
        writer.close()

    # --- HANDLERS ---

    def handle_init(self):
        quest_manager.load_quests()
        current_quest = quest_manager.get_current_quest()
        if current_quest:
            # Check if first objective is not completed. This prevents re-triggering intro on daemon reconnect.
            if current_quest.objectives and not current_quest.objectives[0].completed:
                # Assuming on_quest_start is a script that runs dialogue
                for action_data in current_quest.on_quest_start:
                    from supershell.game import (
                        actions,  # Defer import to avoid circular dependency
                    )

                    actions.run_action(action_data)
            else:
                dialogue.say("System re-connected. Ready.", character="system")
        else:
            dialogue.say("No quests loaded. Daemon ready.", character="system")

    def handle_pre_exec(self, args):
        """Returns 0 to allow, 1 to block."""
        cmd = args[0]
        # Future: Check if command violates constraints (e.g., active_obj.fail_type for blocking)
        # For now, always allow pre-exec unless an explicit block is implemented.
        log.debug(f"Pre-exec check for: {cmd}. Always allowing for now.")
        return 0

    def handle_post_exec(self, args):
        """Checks objective completion and failure conditions."""
        cmd_line = args[0]
        try:
            ret_code = int(args[1])
        except (ValueError, IndexError):
            ret_code = 0  # Default to success if return code is not provided or invalid

        res = CommandResult(command=cmd_line, return_code=ret_code)

        current_quest = quest_manager.get_current_quest()
        if not current_quest:
            return  # No active quest, nothing to check

        # Use the updated quest_validator to get a tri-state status
        status, obj_id = quest_validator.check(res)

        if status == "SUCCESS":
            if obj_id:
                quest_manager.mark_objective_complete(obj_id)
                current_quest.handle_event(obj_id, res)  # obj.completed will be True
        elif status == "FAIL":
            if obj_id:
                current_quest.handle_event(obj_id, res)  # obj.completed will be False
        # If status is "CONTINUE", no specific action is needed here.

    def handle_game_cmd(self, args):
        """Handles quest, hint, etc."""
        if not args:
            dialogue.say("Usage: game_cmd <command> [params...]", character="system")
            return

        cmd = args[0]
        # params = args[1:] # Not currently used, but keep for future expansion

        if cmd == "quest":
            q = quest_manager.get_current_quest()
            if q:
                dialogue.say(
                    f"[bold]{q.title}[/bold]: {q.description}", character="quest"
                )
                for obj in q.objectives:
                    if obj.completed:
                        dialogue.say(
                            f"[dim]✔ {obj.description}[/dim]", character="quest"
                        )
                    else:
                        dialogue.say(
                            f"[bold]➜ {obj.description}[/bold]", character="quest"
                        )
                        break
            else:
                dialogue.say("No active quest.", character="quest")
        elif cmd == "hint":
            hint = quest_manager.get_contextual_hint()
            dialogue.say(hint, character="cypher")
        # Add other commands (ready, foundit) logic here...


async def main():
    if len(sys.argv) < 2:
        print("Usage: daemon.py <socket_path>")
        sys.exit(1)

    socket_path = sys.argv[1]

    # Cleanup old socket
    if os.path.exists(socket_path):
        os.remove(socket_path)
        log.info(f"Cleaned up old socket: {socket_path}")

    # Initialize quest manager here, once.
    # This must happen before any client connections are handled so state is ready.
    quest_manager.load_quests()

    game_server_instance = GameServer()
    server = await asyncio.start_unix_server(
        game_server_instance.handle_client, path=socket_path
    )

    log.info(f"Daemon listening on {socket_path}")

    # Signal handler for graceful shutdown
    loop = asyncio.get_running_loop()
    for sig_name in ("SIGINT", "SIGTERM"):
        loop.add_signal_handler(
            getattr(signal, sig_name),
            lambda: asyncio.create_task(shutdown(server, socket_path)),
        )

    async with server:
        await server.serve_forever()


async def shutdown(server, socket_path):
    log.info("Shutting down daemon gracefully...")
    server.close()
    await server.wait_closed()
    if os.path.exists(socket_path):
        os.remove(socket_path)
    log.info("Daemon shutdown complete.")
    sys.exit(0)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        log.info("Daemon received KeyboardInterrupt. Shutting down gracefully.")
    except Exception as e:
        log.critical(f"Daemon encountered a critical error: {e}", exc_info=True)
        sys.exit(1)
