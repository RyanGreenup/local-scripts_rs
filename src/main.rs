use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// This is a collection of scripts to do amazing things
/// On this system all shell scripts should be migrated into this tool
/// or python under ~/.local/scripts/{python,rust}.
/// Scripts are in modules but one module, one language, all the note stuff
/// Is in python right now because it mostly just shells out.

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },

    Notes {
        /// Use Absolute paths
        #[arg(short, long)]
        relative: bool,

        #[command(subcommand)]
        command: Option<NotesCommands>,
    },
}

#[derive(Subcommand)]
enum NotesCommands {
    List {
        /// Exlcude Journal notes
        #[arg(short, long)]
        exclude_journal: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Notes {
            relative,
            command,
        }) => {
            let mut files = get_notes(*relative);

            match command {
                Some(NotesCommands::List { exclude_journal }) => {
                    if *exclude_journal {
                        // filter out anything with journal in the name
                        files = files
                            .into_iter()
                            .filter(|f| !f.contains("/journal"))
                            .filter(|f| !f.contains(".journal"))
                            .collect();
                    }
                    for file in files {
                        println!("{}", file);
                    }
                }
                None => {}
            }
        }
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {}
    }

    // Continued program logic goes here...
}

fn get_notes_dir() -> String {
    let home = std::env::var("HOME").expect("No $HOME variable found");
    format!("{home}/Notes/slipbox")
}

fn get_notes(relative: bool) -> Vec<String> {
    let mut notes = vec![];
    for entry in walkdir::WalkDir::new(get_notes_dir()) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            if relative {
                notes.push(
                    path.strip_prefix(get_notes_dir())
                        .unwrap()
                        .display()
                        .to_string(),
                );
            } else {
                notes.push(path.display().to_string());
            }
        }
    }
    notes
}
