// shell.rs

use directories::UserDirs;
use log::info;
use std::io::Write;
use std::process::Command;
use tempfile::Builder;

// This Bash script is injected into the user's session.
// It creates the "Prompt" and the "Hooks".
const SHELL_RC_TEMPLATE: &str = r#"
if [ -z "$PS1" ]; then
   return
fi

# 1. THE PROMPT (Cyberpunk Style)
# \w = current working directory
# \e[...m = Color codes
export PS1="\[\e[1;37m\][ \[\e[1;36m\]$USER@supershell\[\e[1;37m\] ] \[\e[1;35m\]\w\[\e[1;37m\] $ \[\e[0m\]"

# 2. THE HOOK FUNCTION (_g)
# This function runs BEFORE every command alias below.
function _g() {
    local cmd=$1
    shift

    # A. Run the actual command (ls, cd, etc.) so the user sees real output
    command "$cmd" "$@"

    # B. Run our Rust binary to check if the user completed a quest
    "__BINARY_PATH__" --check "$cmd"
}

# 3. ALIASES (The Trap)
# When user types 'ls', they actually run '_g ls'
alias ls="_g ls"
alias cd="_g cd"
alias cat="_g cat"
alias pwd="_g pwd"
alias status="__BINARY_PATH__ --status"
alias exit="command exit"
"#;

/// Starts the game environment
pub fn start_shell() {
    // 1. Find where THIS binary is running
    let current_exe = std::env::current_exe()
        .expect("Failed to get self path")
        .to_string_lossy()
        .into_owned();

    // 2. Setup the "Sandbox" (The Construct folder)
    let user_dirs = UserDirs::new().expect("Error: Could not determine home directory.");
    let construct_path = user_dirs.home_dir().join("Construct");

    if !construct_path.exists() {
        std::fs::create_dir_all(&construct_path).expect("Failed to create Construct dir");
    }

    // 3. Inject our Binary Path into the Bash script
    let rc_content = SHELL_RC_TEMPLATE.replace("__BINARY_PATH__", &current_exe);

    // 4. Create a temporary RC file to hold the script
    let mut temp_rc = Builder::new()
        .prefix("construct_rc_")
        .suffix(".bash")
        .rand_bytes(5)
        .tempfile()
        .expect("Failed to create temp RC file");

    write!(temp_rc, "{}", rc_content).expect("Failed to write RC file");

    // 5. Welcome Message
    println!(">> NEURAL LINK ESTABLISHED.");
    println!(">> WELCOME TO THE CONSTRUCT.");
    println!("   (Type 'exit' to disconnect)\n");

    info!("Uplink Established. Handing control to Bash subprocess.");

    // 6. Launch Bash with our custom config
    // --noprofile: Don't load user's real .bashrc (Keep it clean)
    // --rcfile: Load OUR script instead
    let _status = Command::new("bash")
        .current_dir(&construct_path)
        .env("CONSTRUCT_UPLINK", "1")
        .arg("--noprofile")
        .arg("--rcfile")
        .arg(temp_rc.path())
        .status()
        .expect("Failed to launch shell");

    info!("Uplink Severed. Bash subprocess terminated.");
    println!("\n>> [SYSTEM] Link Severed.");
}
