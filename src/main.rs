mod app;
mod editor;
mod headless;
mod paths;
mod popups;
mod preview;
mod profile;
mod rsync;
mod run;
mod screens;
mod store;
mod ui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lazyrsync", version, about = "A terminal UI for rsync")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Run a profile's tasks without the TUI")]
    Run {
        #[arg(
            value_name = "PROFILE[/TASK]",
            help = "Profile to run, or PROFILE/TASK for a single task (ids come from `list`)"
        )]
        target: String,

        #[arg(
            short = 'n',
            long,
            help = "Trial run: report what would transfer, change nothing"
        )]
        dry_run: bool,

        #[arg(
            long,
            help = "Allow tasks that delete files at the destination (not needed with -n)"
        )]
        yes: bool,
    },

    #[command(about = "List profiles, task ids, and their resolved rsync commands")]
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::List) => {
            let store = match store::Store::load(false) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: {e:#}");
                    std::process::exit(2);
                }
            };
            headless::list(&store.profiles);
            Ok(())
        }
        Some(Command::Run {
            target,
            dry_run,
            yes,
        }) => {
            let store = match store::Store::load(false) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: {e:#}");
                    std::process::exit(2);
                }
            };
            std::process::exit(headless::run(&store.profiles, &target, dry_run, yes));
        }
        None => {
            let mut app = app::App::new()?;
            let mut terminal = ratatui::init();
            let prev_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let _ = crossterm::execute!(
                    std::io::stdout(),
                    crossterm::event::DisableBracketedPaste,
                    crossterm::event::DisableMouseCapture,
                );
                prev_hook(info);
            }));
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::event::EnableMouseCapture,
                crossterm::event::EnableBracketedPaste,
            );
            let result = app.run(&mut terminal);
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::event::DisableBracketedPaste,
                crossterm::event::DisableMouseCapture,
            );
            ratatui::restore();
            result
        }
    }
}
