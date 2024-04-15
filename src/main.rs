use clap::{Parser, Subcommand};
use duct::cmd;
use std::path::PathBuf;

mod utils;
mod wm;

use wm::take_screenshot;
use wm::take_screenshot::get_clipboard;
use wm::take_screenshot::set_clipboard;

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

    Wm {
        #[command(subcommand)]
        command: Option<WmCommands>,
    },

    Notes {
        /// Use Absolute paths
        #[arg(short, long)]
        relative: bool,

        /// The name of the execubale to use as the editor
        #[arg(short, long)]
        editor: Option<String>,

        #[command(subcommand)]
        command: Option<NotesCommands>,
    },
}

#[derive(Subcommand)]
enum WmCommands {
    /// Take a screenshot
    Screenshot {
        /// The name of the output file
        #[arg(short, long)]
        output: Option<String>,

        /// Copies the screenshot to the clipboard
        #[arg(short, long)]
        clipboard: bool,
    },
}

#[derive(Subcommand)]
enum NotesCommands {
    /// List Notes
    List {
        /// Exlcude Journal notes
        #[arg(short, long)]
        exclude_journal: bool,
    },

    /// use Fzf to select a note and open in $EDITOR
    Find {},

    /// Create a Subpage under source from a title in the clipboard
    /// e.g. `slipbox notes subpage --source /path/to/source.md`
    /// If the clipboard contains "Title of the Page" it will create a file
    /// "/path/to/source.title-of-the-page.md"
    SubPage {
        /// The Note the link will be inserted to
        #[arg(short, long)]
        source: String,
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
        Some(Commands::Wm { command }) => {
            match command {
                Some(WmCommands::Screenshot { output, clipboard }) => {
                    // TODO this should just be take_screenshot().
                    take_screenshot::main(output.clone(), *clipboard);
                }
                None => {}
            }
        }
        Some(Commands::Notes {
            relative,
            command,
            editor,
        }) => {
            let mut files = get_notes(*relative);

            let editor = match editor {
                Some(editor) => {
                    // Use the provided editor (TODO check if binary exists?)
                    editor.clone()
                }
                None => {
                    // Get editor from the EDITOR var
                    std::env::var("EDITOR").unwrap_or("nvim".to_string())
                }
            };

            match command {
                Some(NotesCommands::Find {}) => {
                    let selected = notes_fzf(*relative);
                    if selected.len() > 0 {
                        let selected = selected[0].clone();
                        let selected = format!("{}/{}", get_notes_dir(), selected);
                        println!("{}", selected);
                        cmd!(editor, selected).run().expect("Failed to open editor");
                    }
                }
                Some(NotesCommands::SubPage { source }) => {
                    let title = get_clipboard().unwrap_or_else(|| {
                        panic!("Failed to get clipboard contents");
                    });
                    let source_file = std::path::Path::new(&source);
                    let ext = source_file
                        .extension()
                        .unwrap_or_else(|| {
                            panic!("Failed to get extension of {:#?}", source_file);
                        })
                        .to_str()
                        .unwrap_or_else(|| {
                            panic!("Failed to convert extension to string {:#?}", source_file);
                        });
                    let filename = title_to_filename(&title, ext);
                    // combine the filename with the source directory
                    let source_dir = source_file.parent().unwrap_or_else(|| {
                        panic!("Failed to get parent directory of {:#?}", source_file);
                    });
                    // strip extension off basename
                    let root = source_file
                        .file_stem()
                        .unwrap_or_else(|| {
                            panic!("Failed to get file stem of {:#?}", source_file);
                        })
                        .to_str()
                        .unwrap_or_else(|| {
                            panic!("Failed to convert file stem to string {:#?}", source_file);
                        });
                    let filename = format!("{root}.{filename}");
                    let filename = source_dir.join(filename);
                    let filename = filename.display().to_string();
                    let link = format!("[{title}]({filename})");
                    set_clipboard(link.clone()).unwrap_or_else(|e| {
                        eprintln!("Failed to set clipboard contents");
                        eprintln!("{}", e);
                    });
                    println!("{}", link);
                }
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

/// Use fzf to select a note and open in $EDITOR
/// TODO abstract this so it can be used in other places
fn notes_fzf(relative: bool) -> Vec<String> {
    let notes = get_notes(relative);
    let notes = notes.join("\n");
    let notes_dir = get_notes_dir();
    let preview_cmd = format!("bat {}/{{}} --color=always --style=snip", notes_dir);
    let chooser = "fzf"; // sk or fzf or fzy
    let fzf = cmd!(chooser, "--preview", preview_cmd)
        .stdin_bytes(notes.as_bytes())
        .read();
    fzf.unwrap().split("\n").map(|s| s.to_string()).collect()
}

fn split_basename(path: &str) -> (std::path::PathBuf, String) {
    let path = std::path::Path::new(path);
    let name = path
        .file_name()
        .unwrap_or_else(|| panic!("Failed to get file name from path {:#?}", path))
        .to_str()
        .unwrap_or_else(|| panic!("Failed to convert file name to string {:#?}", path))
        .to_string();
    let dir = path
        .parent()
        .unwrap_or_else(|| panic!("Failed to get parent directory of {:#?}", path))
        .to_path_buf();
    (dir, name)
}

fn title_to_filename(title: &str, ext: &str) -> String {
    let mut filename: String = title.to_owned().clone();
    // Title to filename
    filename = filename.replace(" / ", ".");
    for bad in vec![" ", ":", ",", "."] {
        filename = filename.replace(bad, "-");
    }
    filename = filename.replace("/", ".");
    filename = format!("{filename}.{ext}");
    filename
}
