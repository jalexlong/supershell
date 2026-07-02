use anyhow::Context;
use directories::UserDirs;
use std::io::Write;
use std::process::Command;
use tempfile::Builder;

// --- THE CYBERPUNK CONFIGURATION ---
const SHELL_RC_TEMPLATE: &str = r#"
# No-Op for standard input to prevent 'stdin: is not a tty' errors
if [ -z "$PS1" ]; then
   return
fi

# 1. ACCESSIBLE CYBERPUNK PROMPT
# Format: [ USER :: CONSTRUCT ] ~/current/path $
# Colors:
# - Brackets: Bold White (Structure)
# - User: Cyan (Identity)
# - Separator: Dark Grey
# - Path: Magenta (Location)
# - Prompt: Bold White
export PS1="\[\e[1;37m\][ \[\e[1;36m\]user\[\e[1;37m\]::\[\e[1;36m\]construct\[\e[1;37m\] ] \[\e[1;35m\]\w\[\e[1;37m\] $ \[\e[0m\]"

# 2. PATH SETUP
export PATH=$PATH:/bin:/usr/bin:/usr/local/bin
export HISTFILE

# 3. THE GUARD (INTERCEPTOR)
# Usage: _g <command> <args>
# This function asks Rust for permission before running the command.
function _g() {
    local cmd=$1
    shift

    # A. Run the User's Command FIRST
    # We let the command run directly so the user sees standard output (e.g., file lists)
    command "$cmd" "$@"

    # Flush in-memory history so HistoryContains can read the current command
    history -a

    # B. Run the Game Check (Directly to Terminal)
    # We do NOT capture output. This allows Rust to handle input/output interactively.
    "__BINARY_PATH__" --check "$cmd $*" --cwd "$PWD"
    local _game_signal=$?

    # C. Check the Signal
    # 2 = task complete → clear and show the next objective
    if [ $_game_signal -eq 2 ]; then
        clear
        "__BINARY_PATH__" --refresh
    fi
}

# 4. GAME ALIASES
alias status='"__BINARY_PATH__" --status'
alias menu='"__BINARY_PATH__" --menu'
alias supershell='"__BINARY_PATH__"'
function help() {
    echo -e "\n\e[1;37m  :: SYSTEM COMMANDS ::\e[0m"
    echo -e "  \e[1;36mstatus\e[0m      - Display current objective."
    echo -e "  \e[1;36mmenu\e[0m        - Return to module selection."
    echo -e "  \e[1;36mexit\e[0m        - Disconnect from the Construct."
    echo ""
}

# 5. THE INFECTION (PUZZLE HOOKS)
# We only hook commands relevant to puzzles.
alias ls='_g ls'
alias cd='_g cd'
alias cat='_g cat'
alias chmod='_g chmod'
alias grep='_g grep'
alias ssh='_g ssh'
alias nano='_g nano'
alias vim='_g vim'

# 6. STARTUP SEQUENCE
clear
echo -e "\n\e[1;36m>> NEURAL LINK ESTABLISHED.\e[0m"
echo -e "\e[1;37m>> WELCOME TO THE CONSTRUCT.\e[0m"
echo -e "\e[0;90m   (Type 'exit' to disconnect)\e[0m\n"

# Trigger the initial game state check
"__BINARY_PATH__" --refresh
"#;

pub fn launch_infected_session() -> anyhow::Result<()> {
    // 1. Check for nesting
    if std::env::var("CONSTRUCT_UPLINK").is_ok() {
        println!("\n\x1b[1;31m>> [ERROR] NEURAL LINK ALREADY ACTIVE.\x1b[0m");
        println!(
            "\x1b[0;90m  (You are already inside the Construct. Type 'exit' to leave.)\x1b[0m\n"
        );
        return Ok(());
    }

    // 2. Get our own executable path
    let current_exe = std::env::current_exe()
        .context("Failed to get executable path")?
        .to_string_lossy()
        .into_owned();

    // 3. Resolve "~/Construct" to an absolute path
    let user_dirs = UserDirs::new().context("Could not determine home directory")?;
    let construct_path = user_dirs.home_dir().join("Construct");

    // Safety check: Ensure the directory exists before dropping the user in.
    if !construct_path.exists() {
        std::fs::create_dir_all(&construct_path).context("Failed to create Construct dir")?;
    }

    // 4. Inject path into the template
    let rc_content = SHELL_RC_TEMPLATE.replace("__BINARY_PATH__", &current_exe);

    // 5. Create a temporary RC file
    let mut temp_rc = Builder::new()
        .prefix("construct_rc_")
        .suffix(".bash")
        .rand_bytes(5)
        .tempfile()
        .context("Failed to create temp RC file")?;

    write!(temp_rc, "{}", rc_content).context("Failed to write RC file")?;

    // 6. Spawn the Shell
    let status = Command::new("bash")
        .current_dir(&construct_path)
        .env("CONSTRUCT_UPLINK", "1")
        .arg("--noprofile")
        .arg("--rcfile")
        .arg(temp_rc.path())
        .status()
        .context("Failed to launch shell")?;

    // 7. Cleanup Message
    if status.success() {
        println!("\n>> [SYSTEM] Link Severed.");
    } else {
        println!("\n>> [SYSTEM] Connection Lost.");
    }

    Ok(())
}
