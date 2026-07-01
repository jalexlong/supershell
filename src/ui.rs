use crossterm::{
    cursor::{MoveDown, MoveToColumn, MoveUp},
    event::{Event, KeyCode, KeyEventKind, poll, read},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};
use std::env;
use std::io::{Write, stdout};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use textwrap::fill;

// --- VISUAL CONSTANTS ---
const BORDER_COLOR: Color = Color::White;
const TITLE_COLOR: Color = Color::White;
const TEXT_COLOR: Color = Color::White;

const MAX_WIDTH: u16 = 80;

// --- PUBLIC API ---

pub fn play_cutscene(text: &str) {
    render_inline_card("MISSION BRIEF", vec![text.to_string()], true);
}

pub fn draw_status_card(
    title: &str,
    chapter: &str,
    instruction: &str,
    objective: &str,
    current: usize,
    total: usize,
) {
    let content = vec![
        format!("📂 {}", chapter),
        String::from(""),
        format!("🎯 OBJECTIVE: {}", objective),
        String::from(""),
        format!("💡 HINT: {}", instruction),
        String::from(""),
        format!("[Progress: {}/{}]", current, total),
    ];
    render_inline_card(title, content, false);
}

pub fn print_success(msg: &str) {
    let mut stdout = stdout();
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("\r\n>> [SUCCESS] "),
        SetAttribute(Attribute::Reset),
        SetForegroundColor(Color::White),
        Print(msg),
        Print("\n\n")
    )
    .ok();
}

pub fn print_fail(error: &str, hint: &str) {
    let mut stdout = stdout();

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("[+] SYSTEM_INTEGRITY.. OK\n"),
        Print("[+] SYNTAX_VALIDATION. OK\n"),
        SetForegroundColor(Color::Red),
        Print("[-] EXECUTION......... FAIL\n"),
        Print("    └── "),
        Print(error),
        Print("\n\n"),
    )
    .ok();

    if !hint.is_empty() {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(" >> HINT: "),
            SetAttribute(Attribute::Bold),
            Print(hint),
            SetAttribute(Attribute::Reset),
            Print("\n"),
        )
        .ok();
    }
    execute!(stdout, ResetColor).ok();
}

/// Display a module selection menu and return the chosen course path.
///
/// - If no modules exist, prints an error and returns None.
/// - If only one module exists (or SUPERSHELL_TEST_MODE is set), auto-selects it.
/// - Otherwise, shows an arrow-key menu in raw mode; falls back to numbered
///   input if raw mode is unavailable.
pub fn show_module_menu(courses: Vec<(PathBuf, String)>) -> Option<PathBuf> {
    if courses.is_empty() {
        eprintln!(">> [ERROR] No modules found.");
        return None;
    }

    // Auto-select when there is no real choice, or in test mode (no stdin).
    if courses.len() == 1 || env::var("SUPERSHELL_TEST_MODE").is_ok() {
        println!("\n>> [MODULE] Auto-selected: {}", courses[0].1);
        return Some(courses[0].0.clone());
    }

    if enable_raw_mode().is_ok() {
        let result = arrow_key_menu(&courses);
        disable_raw_mode().ok();
        result
    } else {
        numbered_menu(&courses)
    }
}

// --- CORE RENDERING ENGINE ---

fn render_inline_card(title: &str, raw_lines: Vec<String>, use_typewriter: bool) {
    // Bypass for test environments.
    if env::var("SUPERSHELL_TEST_MODE").is_ok() {
        render_plain_card(title, &raw_lines, false);
        return;
    }

    // Enter raw mode; fall back to plain renderer if unavailable.
    if enable_raw_mode().is_err() {
        render_plain_card(title, &raw_lines, use_typewriter);
        return;
    }
    let mut stdout = stdout();

    let (term_cols, term_rows) = size().unwrap_or((80, 24));

    // Fall back to plain renderer for constrained terminals.
    if term_cols < 50 || term_rows < 16 {
        disable_raw_mode().ok();
        render_plain_card(title, &raw_lines, use_typewriter);
        return;
    }

    let width = std::cmp::min(term_cols, MAX_WIDTH);
    let content_width = (width as usize).saturating_sub(4);

    let mut final_lines = Vec::new();
    for raw_line in raw_lines {
        if raw_line.is_empty() {
            final_lines.push("".to_string());
        } else {
            let wrapped = fill(&raw_line, content_width);
            for subline in wrapped.split('\n') {
                final_lines.push(subline.to_string());
            }
        }
    }

    let height = final_lines.len();

    // Draw skeleton.
    print_top_border(width, title);

    execute!(stdout, SetForegroundColor(BORDER_COLOR)).ok();
    for _ in 0..height {
        print!("│");
        print!("{}", " ".repeat(content_width + 2));
        execute!(stdout, MoveToColumn(width - 1)).ok();
        print!("│\r\n");
    }
    print_bottom_border(width);

    // Rewind to top of content area.
    execute!(stdout, MoveUp((height + 1) as u16)).ok();

    // Fill content.
    let mut skipped = false;

    for line in final_lines {
        execute!(stdout, MoveToColumn(2)).ok();
        execute!(stdout, SetForegroundColor(TEXT_COLOR)).ok();

        if use_typewriter {
            for char in line.chars() {
                print!("{}", char);
                stdout.flush().ok();

                if !skipped {
                    if poll(Duration::from_secs(0)).unwrap_or(false) {
                        if let Ok(Event::Key(key)) = read() {
                            if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                                skipped = true;
                            }
                        }
                    }
                    if !skipped {
                        thread::sleep(Duration::from_millis(15));
                    }
                }
            }
        } else {
            print!("{}", line);
        }

        execute!(stdout, MoveDown(1)).ok();
    }

    // Move past the bottom border.
    execute!(stdout, MoveDown(1)).ok();
    execute!(stdout, MoveToColumn(0), ResetColor).ok();

    if use_typewriter {
        print!("\r\n>> PRESS [ENTER] TO CONTINUE...");
        stdout.flush().ok();

        loop {
            match read() {
                Ok(Event::Key(key))
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter =>
                {
                    break;
                }
                Err(_) => break,
                _ => {}
            }
        }
        print!("\r\n");
    } else {
        print!("\r\n");
    }

    // CRITICAL: always restore the terminal before returning.
    disable_raw_mode().ok();
}

fn render_plain_card(title: &str, raw_lines: &[String], use_pause: bool) {
    println!();
    println!("== {} ==", title);

    for raw_line in raw_lines {
        if raw_line.is_empty() {
            println!();
            continue;
        }
        println!("{}", raw_line);
    }

    if use_pause {
        println!();
        println!(">> PRESS [ENTER] TO CONTINUE...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    } else {
        println!();
    }
}

// --- MODULE MENU ---

/// Arrow-key driven menu. Caller is responsible for enable/disable raw mode.
fn arrow_key_menu(courses: &[(PathBuf, String)]) -> Option<PathBuf> {
    let n = courses.len();
    let mut selected = 0usize;
    let mut stdout = stdout();

    // Static header (printed once; not part of the re-render region).
    execute!(
        stdout,
        Print("\r\n"),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::White),
        Print(">> MODULE SELECT\r\n"),
        ResetColor,
        Print("\r\n"),
    )
    .ok();

    // Initial render.
    print_menu_items(courses, selected, &mut stdout);

    loop {
        match read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                        rerender_menu(courses, selected, n, &mut stdout);
                    }
                }
                KeyCode::Down => {
                    if selected + 1 < n {
                        selected += 1;
                        rerender_menu(courses, selected, n, &mut stdout);
                    }
                }
                KeyCode::Enter => {
                    execute!(stdout, Print("\r\n"), ResetColor).ok();
                    return Some(courses[selected].0.clone());
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    execute!(stdout, Print("\r\n"), ResetColor).ok();
                    return None;
                }
                _ => {}
            },
            Err(_) => return None,
            _ => {}
        }
    }
}

/// Erase the rendered item list and footer, then redraw with the new selection.
///
/// Layout printed by print_menu_items:
///   n item lines  (each ends with \r\n)
///   1 blank line  (\r\n)
///   footer line   (no trailing newline — cursor sits here)
///
/// Total lines below item-1 start: n + 1 full lines, cursor on line n+2.
/// Moving up n+1 lines returns to the start of item 1.
fn rerender_menu(
    courses: &[(PathBuf, String)],
    selected: usize,
    n: usize,
    stdout: &mut std::io::Stdout,
) {
    execute!(stdout, MoveToColumn(0)).ok();
    execute!(stdout, MoveUp((n + 1) as u16)).ok();
    execute!(stdout, Clear(ClearType::FromCursorDown)).ok();
    print_menu_items(courses, selected, stdout);
}

/// Render the item list and footer. Does NOT print the static header.
fn print_menu_items(courses: &[(PathBuf, String)], selected: usize, stdout: &mut std::io::Stdout) {
    for (i, (_, name)) in courses.iter().enumerate() {
        if i == selected {
            execute!(
                stdout,
                SetForegroundColor(Color::Cyan),
                SetAttribute(Attribute::Bold),
                Print(format!("  \u{276f} {name}\r\n")),
                ResetColor,
            )
            .ok();
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(format!("    {name}\r\n")),
                ResetColor,
            )
            .ok();
        }
    }
    execute!(stdout, Print("\r\n")).ok(); // blank line before footer
    execute!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print("  \u{2191}\u{2193} Navigate   [ENTER] Select   [ESC] Cancel"),
        ResetColor,
    )
    .ok();
    stdout.flush().ok();
}

/// Numbered list fallback for environments where raw mode is unavailable.
fn numbered_menu(courses: &[(PathBuf, String)]) -> Option<PathBuf> {
    println!();
    println!(">> MODULE SELECT");
    println!();
    for (i, (_, name)) in courses.iter().enumerate() {
        println!("  [{}] {}", i + 1, name);
    }
    println!();
    print!("  Enter number: ");
    stdout().flush().ok();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    let choice = input.trim().parse::<usize>().ok()?;
    if choice >= 1 && choice <= courses.len() {
        courses.get(choice - 1).map(|(path, _)| path.clone())
    } else {
        eprintln!(">> [ERROR] Invalid selection.");
        None
    }
}

// --- BORDER HELPERS ---

fn print_top_border(width: u16, title: &str) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).ok();
    print!("┌──");

    execute!(
        stdout,
        SetForegroundColor(TITLE_COLOR),
        SetAttribute(Attribute::Bold)
    )
    .ok();
    print!(" {} ", title);

    execute!(
        stdout,
        SetForegroundColor(BORDER_COLOR),
        SetAttribute(Attribute::Reset)
    )
    .ok();

    let used_len = 3 + 1 + title.chars().count() + 1;
    let remaining = (width as usize).saturating_sub(used_len + 1);

    print!("{}", "─".repeat(remaining));
    execute!(stdout, MoveToColumn(width - 1)).ok();
    print!("┐\r\n");
}

fn print_bottom_border(width: u16) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).ok();
    print!("└{}", "─".repeat((width as usize).saturating_sub(2)));
    execute!(stdout, MoveToColumn(width - 1)).ok();
    print!("┘\r\n");
    execute!(stdout, ResetColor).ok();
}
