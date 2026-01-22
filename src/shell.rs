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

# 3. THE GUARD (INTERCEPTOR)
# Usage: _g <command> <args>
# This function asks Rust for permission before running the command.
function _g() {
    local cmd=$1
    shift # Pop the first argument (the command name)

    # A. Validation
    # We call our own executable with the --check flag.
    # "$*" combines all remaining arguments into a single string.
    "__BINARY_PATH__" --check "$cmd $*"
    local status=$?

    # B. Decision
    if [ $status -eq 0 ]; then
        # Success (Exit 0): The Game allowed it.
        # Run the REAL command, bypassing aliases.
        command "$cmd" "$@"
    else
        # Failure (Exit 1): The Game blocked it.
        # We do nothing. The Rust binary already printed the error UI.
        return $status
    fi
}

# 4. THE INFECTION (ALIASES)
# We only hook commands relevant to puzzles.
alias ls='_g ls'
alias cd='_g cd'
alias cat='_g cat'
alias grep='_g grep'
alias ssh='_g ssh'
alias python='_g python'
alias python3='_g python3'
alias nano='_g nano'
alias vim='_g vim'

# 5. STARTUP BANNER
echo -e "\n\e[1;36m>> NEURAL LINK ESTABLISHED.\e[0m"
echo -e "\e[1;37m>> WELCOME TO THE CONSTRUCT.\e[0m"
echo -e "\e[0;90m   (Type 'exit' to disconnect)\e[0m\n"
"#;

pub fn launch_infected_session() {
    // 1. Get our own executable path
    let current_exe = std::env::current_exe()
        .expect("Failed to get executable path")
        .to_string_lossy()
        .into_owned();

    // 2. Resolve "~/Construct" to an absolute path
    let user_dirs = UserDirs::new().expect("Error: Could not determine home directory.");
    let construct_path = user_dirs.home_dir().join("Construct");

    // Safety check: Ensure the directory exists before dropping the user in.
    if !construct_path.exists() {
        std::fs::create_dir_all(&construct_path).expect("Failed to create Construct dir");
    }

    // 3. Inject path into the template
    let rc_content = SHELL_RC_TEMPLATE.replace("__BINARY_PATH__", &current_exe);

    // 4. Create a temporary RC file
    let mut temp_rc = Builder::new()
        .prefix("construct_rc_")
        .suffix(".bash")
        .rand_bytes(5)
        .tempfile()
        .expect("Failed to create temp RC file");

    write!(temp_rc, "{}", rc_content).expect("Failed to write RC file");

    // 5. Spawn the Shell
    // We use --noprofile to ensure a clean slate
    // We use --rcfile to force our custom config
    // We use .current_dir() to force them into the game world
    let status = Command::new("bash")
        .current_dir(&construct_path)
        .arg("--noprofile")
        .arg("--rcfile")
        .arg(temp_rc.path())
        .status()
        .expect("Failed to launch shell");

    // 6. Cleanup Message
    if status.success() {
        println!("\n>> [SYSTEM] Link Severed.");
    } else {
        println!("\n>> [SYSTEM] Connection Lost.");
    }
}
