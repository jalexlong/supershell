"""
The main command-line entry point for the supershell game.
"""

from supershell.core.game_loop import main_loop

def main():
    """
    Main function called by the 'supershell' command.
    """
    try:
        main_loop()
    except Exception as e:
        print(f"A fatal error occurred: {e}")
        # Optionally, log the full traceback here

if __name__ == "__main__":
    main()
