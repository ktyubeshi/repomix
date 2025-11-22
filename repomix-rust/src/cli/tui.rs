use anyhow::Result;
use console::{style, Key, Term};
use dialoguer::{theme::ColorfulTheme, Select};

const SYMBOL_PIPE: &str = "│";
const SYMBOL_CORNER: &str = "└";
const SYMBOL_TOP: &str = "┌";
const SYMBOL_INFO: &str = "ℹ";
const SYMBOL_SUCCESS: &str = "✔";

#[derive(PartialEq)]
pub enum PromptResult<T> {
    Ok(T),
    Cancel,
}

pub struct Tui {
    term: Term,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    pub fn intro(&self, message: &str) {
        println!();
        println!("{} {}", style(SYMBOL_TOP).cyan(), style(message).bold());
        println!("{}", style(SYMBOL_PIPE).cyan());
    }

    pub fn outro_success(&self, message: &str) {
        println!(
            "{} {} {}",
            style(SYMBOL_CORNER).cyan(),
            style(SYMBOL_SUCCESS).green(),
            style(message).green()
        );
        println!();
    }

    pub fn outro_warning(&self, message: &str) {
        println!(
            "{} {}",
            style(SYMBOL_CORNER).cyan(),
            style(message).yellow()
        );
        println!();
    }

    pub fn cancel_and_exit(&self) -> ! {
        println!(
            "{} {}",
            style(SYMBOL_CORNER).cyan(),
            style("Initialization cancelled.").dim()
        );
        println!();
        std::process::exit(0);
    }

    pub fn log_info(&self, message: &str) {
        println!(
            "{}   {} {}",
            style(SYMBOL_PIPE).cyan(),
            style(SYMBOL_INFO).blue(),
            message
        );
    }

    pub fn log_success(&self, message: &str) {
        println!(
            "{}   {} {}",
            style(SYMBOL_PIPE).cyan(),
            style(SYMBOL_SUCCESS).green(),
            message
        );
    }

    pub fn confirm(&self, message: &str, default_yes: bool) -> Result<PromptResult<bool>> {
        let term = &self.term;
        let mut current = default_yes;
        let mut first_render = true;

        term.hide_cursor()?;

        loop {
            if !first_render {
                term.clear_last_lines(2)?;
            }
            first_render = false;

            let yes_circle = if current {
                style("●").green()
            } else {
                style("○").dim()
            };
            let yes_label = if current {
                style("Yes").green()
            } else {
                style("Yes").dim()
            };
            let no_circle = if !current {
                style("●").green()
            } else {
                style("○").dim()
            };
            let no_label = if !current {
                style("No").green()
            } else {
                style("No").dim()
            };

            term.write_line(&format!(
                "{} {} {}",
                style(SYMBOL_PIPE).cyan(),
                style("◆").magenta(),
                message
            ))?;
            term.write_line(&format!(
                "{}   {} {} {} {}",
                style(SYMBOL_PIPE).cyan(),
                yes_circle,
                yes_label,
                style("/").dim(),
                format!("{} {}", no_circle, no_label)
            ))?;
            term.flush()?;

            match term.read_key()? {
                Key::Enter => {
                    term.show_cursor()?;
                    return Ok(PromptResult::Ok(current));
                }
                Key::ArrowLeft | Key::ArrowUp => current = true,
                Key::ArrowRight | Key::ArrowDown => current = false,
                Key::Char('y') | Key::Char('Y') => {
                    term.show_cursor()?;
                    return Ok(PromptResult::Ok(true));
                }
                Key::Char('n') | Key::Char('N') => {
                    term.show_cursor()?;
                    return Ok(PromptResult::Ok(false));
                }
                Key::Char('\n') => {
                    term.show_cursor()?;
                    return Ok(PromptResult::Ok(current));
                }
                Key::Escape | Key::CtrlC => {
                    term.show_cursor()?;
                    return Ok(PromptResult::Cancel);
                }
                _ => {}
            }
        }
    }

    pub fn select<T: std::fmt::Display>(
        &self,
        prompt: &str,
        items: &[T],
        default: usize,
    ) -> Result<PromptResult<usize>> {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default)
            .items(items)
            .interact_opt()?;

        match selection {
            Some(index) => Ok(PromptResult::Ok(index)),
            None => Ok(PromptResult::Cancel),
        }
    }

    pub fn input<F>(&self, prompt: &str, initial: &str, validate: F) -> Result<PromptResult<String>>
    where
        F: Fn(&str) -> Result<(), String>,
    {
        let term = &self.term;
        let mut value = initial.to_string();
        let mut first_render = true;
        let mut error: Option<String> = None;

        term.hide_cursor()?;

        loop {
            if !first_render {
                let lines_to_clear = 2 + if error.is_some() { 1 } else { 0 };
                term.clear_last_lines(lines_to_clear)?;
            }
            first_render = false;

            term.write_line(&format!(
                "{} {} {}",
                style(SYMBOL_PIPE).cyan(),
                style("◆").magenta(),
                prompt
            ))?;
            term.write_line(&format!("{}   {}", style(SYMBOL_PIPE).cyan(), value))?;

            if let Some(err) = &error {
                term.write_line(&format!(
                    "{}   {}",
                    style(SYMBOL_PIPE).cyan(),
                    style(err).red()
                ))?;
            }

            term.flush()?;

            match term.read_key()? {
                Key::Enter => match validate(&value) {
                    Ok(_) => {
                        term.show_cursor()?;
                        return Ok(PromptResult::Ok(value));
                    }
                    Err(msg) => {
                        error = Some(msg);
                        continue;
                    }
                },
                Key::Escape | Key::CtrlC => {
                    term.show_cursor()?;
                    return Ok(PromptResult::Cancel);
                }
                Key::Backspace => {
                    value.pop();
                }
                Key::Char(c) => {
                    if c != '\n' && c != '\r' {
                        value.push(c);
                    }
                }
                _ => {}
            }
        }
    }
}
