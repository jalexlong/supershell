"""
The Thin Client.
Sends command context to the Daemon and prints the response.
"""

import json
import os
import socket
import sys


def main():
    if len(sys.argv) < 4:
        # Usage: client.py <socket_path> <cwd> <hook_type> [args...]
        # This will be called by supershell.sh
        return

    socket_path = sys.argv[1]
    cwd = sys.argv[2]
    hook_type = sys.argv[3]
    args = sys.argv[4:]

    request = {"cwd": cwd, "type": hook_type, "args": args}

    try:
        # Connect to the Daemon
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.connect(socket_path)

            # Send Request
            client.sendall(json.dumps(request).encode("utf-8"))
            client.shutdown(socket.SHUT_WR)  # Tell server we are done sending

            # Receive Response
            response_data = b""
            while True:
                chunk = client.recv(4096)
                if not chunk:
                    break
                response_data += chunk

            if not response_data:
                # This can happen if the daemon closes the connection without sending data
                sys.exit(0)

            response = json.loads(response_data.decode("utf-8"))

            # 1. Handle Output (Dialogue)
            for line in response.get("output", []):
                # We print directly to stdout so the user sees it
                print(line)

            # 2. Handle Exit Code (for Pre-Exec blocking)
            sys.exit(response.get("exit_code", 0))

    except ConnectionRefusedError:
        print("[!] Error: Supershell Daemon is not running.")
        sys.exit(1)
    except FileNotFoundError:
        print(
            f"[!] Error: Socket file not found at {socket_path}. Is the daemon running?"
        )
        sys.exit(1)
    except Exception as e:
        print(f"[!] Client Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
