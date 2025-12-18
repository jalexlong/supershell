# Supershell 🛡️

> *The terminal is dark and full of errors. Take this.*

Supershell is a gamified CLI tutor designed to teach Bash mastery through an immersive, interactive narrative. Unlike other tutorials that run in a sandbox, Supershell runs in your **real terminal** as you work, rewarding you for performing actual tasks.

## 🏗️ Architecture

Supershell follows a **"Fail-Open"** philosophy. It is designed to never interfere with your critical shell operations.

* **The Host (Bash):** A lightweight hook captures your last command and passes it to the engine. If the engine is missing, the shell keeps working.
* **The Engine (Rust):** A stateless binary that checks your command against active quest criteria (Regex, File System checks).
* **The UI (TUI):** A "Matrix-style" overlay that hijacks the screen *only* when you complete an objective, creating a seamless "Augmented Reality" feel.

## 🚀 Getting Started (Dev Mode)

### Prerequisites
* Rust (2024 Edition)
* Bash

### Installation
1.  Clone the repository:
    ```bash
    git clone [https://github.com/jalexlong/supershell.git](https://github.com/jalexlong/supershell.git)
    cd supershell
    ```
2.  Build the engine:
    ```bash
    cargo build
    ```
3.  Install the data (XDG Standard):
    ```bash
    mkdir -p ~/.local/share/supershell
    cp quests.yaml ~/.local/share/supershell/
    ```
4.  Activate the hook:
    ```bash
    source supershell.sh
    ```

## 🎮 How to Play

1.  Once installed, type `supershell` to initialize the uplink.
2.  Follow the instructions from the daemon.
3.  Type real commands (`ls`, `cd`, `echo`) to solve the puzzles.
4.  The system will interrupt you when a protocol is satisfied.

## 📂 Data Locations (Linux)
Supershell adheres to the XDG Base Directory Specification:
* **Content:** `~/.local/share/supershell/quests.yaml`
* **Save Data:** `~/.local/share/supershell/save.json`

##  license
MIT License
