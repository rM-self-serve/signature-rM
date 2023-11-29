use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::os::unix::fs::FileExt;
use std::process::ExitCode;

pub const XOBIN_PATH: &str = "/usr/bin/xochitl";
pub const RM_CONF: &str = "/usr/share/remarkable/update.conf";
pub const BCKUP_DIR: &str = "/home/root/.local/share/signature-rM";

pub const CLI_ABOUT: &str = "
Remove the signature from the bottom of emails sent from the device. 
Source+Docs: https://github.com/rM-self-serve/signature-rM

Remember to run 'systemctl stop xochitl' before applying/reverting,
then 'systemctl start xochitl' once applied/reverted";

#[derive(Parser)]
#[command(author, version, about = CLI_ABOUT, long_about = None, arg_required_else_help(true))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Apply the modification
    Apply {
        /// Do not prompt for confirmation
        #[arg(short = 'y', long = "no-prompt", action)]
        no_prompt: bool,
    },
    #[command(arg_required_else_help(true))]
    /// Revert the modification
    Revert {
        /// Revert from backup
        #[arg(short, long, action, conflicts_with = "reverse")]
        backup: bool,
        /// Revert by reversing the modification
        #[arg(short, long, action, conflicts_with = "backup")]
        reverse: bool,
        /// Do not prompt for confirmation
        #[arg(short = 'y', long = "no-prompt", action)]
        no_prompt: bool,
    },
    /// Return true or false
    IsApplied {},
    /// Return true or false
    CanApply {},
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Apply { no_prompt }) => {
            if !apply_entry(no_prompt) {
                return ExitCode::FAILURE;
            }
        }
        Some(Commands::Revert {
            backup,
            reverse,
            no_prompt,
        }) => {
            if backup == reverse {
                println!("Select either '--backup' or '--reverse'");
                return ExitCode::FAILURE;
            }
            if *backup && !revert_from_backup_entry(no_prompt) {
                return ExitCode::FAILURE;
            }
            if *reverse && !revert_by_reverse_entry(no_prompt) {
                return ExitCode::FAILURE;
            }
        }
        Some(Commands::IsApplied {}) => {
            let is_applied = is_applied();
            println!("{is_applied}");
            if !is_applied {
                return ExitCode::FAILURE;
            }
        }
        Some(Commands::CanApply {}) => {
            let can_apply = can_apply();
            println!("{can_apply}");
            if !can_apply {
                return ExitCode::FAILURE;
            }
        }
        None => {}
    }

    ExitCode::SUCCESS
}

fn is_applied() -> bool {
    let result = match mod_index() {
        Ok(val) => val,
        Err(err) => {
            println!("{err}");
            return false;
        }
    };

    result.is_some()
}

fn can_apply() -> bool {
    let result = match og_index() {
        Ok(val) => val,
        Err(err) => {
            println!("{err}");
            return false;
        }
    };

    result.is_some()
}

fn og_index() -> std::io::Result<Option<usize>> {
    let xobytes = std::fs::read(XOBIN_PATH)?;

    let guilty = b"Sent from my reMarkable";

    Ok(xobytes
        .windows(guilty.len())
        .position(move |sub| sub == guilty))
}

fn mod_index() -> std::io::Result<Option<usize>> {
    let xobytes = std::fs::read(XOBIN_PATH)?;

    let absolved = b"\0ent from my reMarkable";

    Ok(xobytes
        .windows(absolved.len())
        .position(move |sub| sub == absolved))
}

fn prompt(sub: &str) -> std::io::Result<bool> {
    print!(
        "Are you sure you want to {} the signature modification? (N/y): ",
        sub
    );
    std::io::stdout().flush()?;
    let mut ovrwrt = String::new();
    std::io::stdin().read_line(&mut ovrwrt)?;
    ovrwrt = ovrwrt.replace("\n", "");
    if ovrwrt.to_lowercase() != "y" {
        println!("Cancelled");
        return Ok(false);
    }
    Ok(true)
}

fn apply_entry(no_prompt: &bool) -> bool {
    xo_warn();

    if let Err(err) = apply(no_prompt) {
        println!("{err}");
        return false;
    };
    true
}

fn apply(no_prompt: &bool) -> std::io::Result<()> {
    let vers = get_version()?;
    let bak_file = format!("{BCKUP_DIR}/xochitl-{vers}-bak");
    println!("This will make a backup of xochitl at:\n{}\n", bak_file);

    if !no_prompt && !prompt("apply")? {
        return Ok(());
    }

    let Some(ind) = og_index()? else {
        let err_str;
        if is_applied() {
            err_str = format!("Modification has already been applied");
        } else {
            err_str = format!("File is not recognized");
        }
        return Err(Error::new(ErrorKind::Other, err_str));
    };

    backup(bak_file)?;

    let file = OpenOptions::new().read(true).write(true).open(XOBIN_PATH)?;
    file.write_at(b"\0", ind as u64)?;

    println!("Successfully removed the signature");
    Ok(())
}

fn backup(bak_file: String) -> std::io::Result<()> {
    std::fs::create_dir_all(BCKUP_DIR)?;
    std::fs::copy(XOBIN_PATH, bak_file)?;

    Ok(())
}

fn revert_by_reverse_entry(no_prompt: &bool) -> bool {
    xo_warn();

    if let Err(err) = revert_by_reverse(no_prompt) {
        println!("{err}");
        return false;
    };

    true
}

fn revert_by_reverse(no_prompt: &bool) -> std::io::Result<()> {
    if !no_prompt && !prompt("revert")? {
        return Ok(());
    }
    let Some(ind) = mod_index()? else {
        let err_str;
        if can_apply() {
            err_str = format!("Modification has not been applied");
        } else {
            err_str = format!("File is not recognized");
        }

        return Err(Error::new(ErrorKind::Other, err_str));
    };

    let file = OpenOptions::new().read(true).write(true).open(XOBIN_PATH)?;
    file.write_at(b"S", ind as u64)?;

    println!("Successfully reversed the signature modification");
    Ok(())
}

fn revert_from_backup_entry(no_prompt: &bool) -> bool {
    xo_warn();

    if let Err(err) = revert_from_backup(no_prompt) {
        println!("{err}");
        return false;
    };

    true
}

fn revert_from_backup(no_prompt: &bool) -> std::io::Result<()> {
    if !no_prompt && !prompt("revert")? {
        return Ok(());
    }

    let vers = get_version()?;
    let bak_file = format!("{BCKUP_DIR}/xochitl-{vers}-bak");

    std::fs::copy(bak_file, XOBIN_PATH).map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            Error::new(ErrorKind::Other, "Can not find backup file".to_string())
        } else {
            err
        }
    })?;

    println!("Successfully reverted the signature modification from backup");
    Ok(())
}

fn get_version() -> std::io::Result<String> {
    let conf_str = std::fs::read_to_string(RM_CONF)?;
    let re = Regex::new(r"[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+").unwrap();
    let err_str = format!("Can not find xochitl version");

    for line in conf_str.split('\n') {
        if line.contains("REMARKABLE_RELEASE_VERSION") {
            let Some(vstring) = re.find(&conf_str) else {
                return Err(Error::new(ErrorKind::Other, err_str));
            };

            return Ok(vstring.as_str().to_owned());
        }
    }

    return Err(Error::new(ErrorKind::Other, err_str));
}

fn xo_warn() {
    println!(
        "Remember to run 'systemctl stop xochitl' before applying,
then 'systemctl start xochitl' once applied\n"
    );
}
