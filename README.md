# supershell 📟

**A story-driven, interactive bash tutorial game where you learn real command-line skills to survive in a dystopian world.**

### 📖 The Story

The year is 20XX. The System is everywhere. You are a new operator, lost in the digital noise, with no memory of how you got here.

Your only companion is Cypher, a mysterious digital being who communicates with you through your terminal. Cypher's origins are unknown, but they seem dedicated to helping you "stabilize the signal" and find your way.

To survive, you'll need to learn the system's "supershell" (a.k.a. bash) from the ground up. Cypher will give you quests, teach you to navigate the file system, manage processes, and eventually, understand the very network that holds the world captive.

### 🎮 The Game

supershell is not a simulation. It's a real bash shell wrapper written in Python.

When you type ls -l, you are actually running ls -l. The game intercepts your commands, checks them against your quest objectives, and then lets the real shell do the work. This "learn-by-doing" approach means every skill you learn is a real, transferable skill.

### ✨ Features

- Real Shell Environment: Learn bash by using bash. No fake consoles.

- Immersive Story: A dystopian, cypherpunk world guides your learning.

- Companion Guide: Cypher provides quests, contextual hints, and lore.

- Progressive Curriculum:

- Bash Basics: ls, cd, pwd, mkdir, rm, cp, mv, echo, and more.

- Networking: Learn ip, ping, and understand DHCP, DNS, and subnetting through interactive missions.

- Modern TUI: A beautiful, responsive terminal experience powered by Rich.

- Smart Shell Features: Includes command history and inline auto-suggestions (like the fish shell) powered by prompt_toolkit.

## 🚀 Installation

supershell is built using Poetry and requires Python 3.13+.

Clone the repository:
```bash
git clone https://github.com/jalexlong/supershell.git
cd supershell
```

Install dependencies using Poetry:
```bash
poetry install
```
## 🕹️ How to Play

Activate the virtual environment:
```bash
eval "$(poetry env activate)"
```

Run the game:
```bash
supershell
```

That's it! Cypher will greet you and your first quest will begin.

## 🤝 Contributing

Contributions are welcome! This is a passion project, and help is always appreciated.

Please feel free to:

- Open an issue to report a bug or suggest a feature.

- Fork the repo and submit a pull request.

- Add new quest ideas to the assets/quests/ directory.

For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the MIT License - see the LICENSE.md file for details.
