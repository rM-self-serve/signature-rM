use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, Write};
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use sysinfo::{DiskExt, System, SystemExt};

pub const XOBIN_PATH: &str = "/usr/bin/xochitl";
pub const RM_CONF: &str = "/usr/share/remarkable/update.conf";
pub const BCKUP_DIR: &str = "/home/root/.local/share/signature-rM";
pub const TMP_FILE: &str = "/home/root/.local/share/signature-rM/signature-rm.xochitl.tmp";

pub const CLI_ABOUT: &str = "
Remove the signature from the bottom of emails sent from the device. 
Source+Docs: https://github.com/rM-self-serve/signature-rM

Remember to run the following once applied/reverted:
$ systemctl restart xochitl";

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
    /// Return true or false
    HasBackup {},
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Apply { no_prompt }) => {
            if !check_space() {
                return ExitCode::FAILURE;
            }
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
            if !check_space() {
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
        Some(Commands::HasBackup {}) => {
            let has_backup = has_backup();
            println!("{has_backup}");
            if !has_backup {
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

fn has_backup() -> bool {
    let vers = match get_version() {
        Ok(val) => val,
        Err(err) => {
            println!("{err}");
            return false;
        }
    };
    let bak_file = format!("{BCKUP_DIR}/xochitl-{vers}-hacked-bak");
    Path::new(&bak_file).exists()
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
    if let Err(err) = apply(no_prompt) {
        println!("{err}");
        return false;
    };
    true
}

fn apply(no_prompt: &bool) -> std::io::Result<()> {
    if !no_prompt && !prompt("apply")? {
        return Ok(());
    }

    let vers = get_version()?;
    let bak_file = format!("{BCKUP_DIR}/xochitl-{vers}-bak");
    println!("This will make a backup of xochitl at:\n{}\n", bak_file);

    if is_applied() {
        println!("Modification has already been applied");
        return Ok(());
    }
    let Some(ind) = og_index()? else {
        let err_str = format!("File is not recognized");
        return Err(Error::new(ErrorKind::Other, err_str));
    };

    backup(bak_file)?;

    std::fs::copy(XOBIN_PATH, TMP_FILE)?;
    let file = OpenOptions::new().read(true).write(true).open(TMP_FILE)?;
    file.write_at(b"\0", ind as u64)?; // the entire hack
    cmd_cp(TMP_FILE, XOBIN_PATH)?;
    std::fs::remove_file(TMP_FILE)?;

    println!("Successfully removed the signature");
    Ok(())
}

fn backup(bak_file: String) -> std::io::Result<()> {
    std::fs::create_dir_all(BCKUP_DIR)?;
    std::fs::copy(XOBIN_PATH, bak_file)?;

    Ok(())
}

fn revert_by_reverse_entry(no_prompt: &bool) -> bool {
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

    let vers = get_version()?;
    let bak_file = format!("{BCKUP_DIR}/xochitl-{vers}-hacked-bak");
    println!(
        "This will make a backup of the modified xochitl binary at:\n{}\n",
        bak_file
    );

    let Some(ind) = mod_index()? else {
        println!("Modification has not been applied");
        return Ok(());
    };

    backup(bak_file)?;

    std::fs::copy(XOBIN_PATH, TMP_FILE)?;
    let file = OpenOptions::new().read(true).write(true).open(TMP_FILE)?;
    file.write_at(b"S", ind as u64)?;
    cmd_cp(TMP_FILE, XOBIN_PATH)?;
    std::fs::remove_file(TMP_FILE)?;

    println!("Successfully reversed the signature modification");
    Ok(())
}

fn revert_from_backup_entry(no_prompt: &bool) -> bool {
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

    if mod_index()?.is_none() {
        println!("Modification has not been applied");
        return Ok(());
    };

    let vers = get_version()?;
    let bak_file = format!("{BCKUP_DIR}/xochitl-{vers}-bak");

    if !Path::new(&bak_file).exists() {
        let err_str = "Can not find backup file".to_string();
        return Err(Error::new(ErrorKind::Other, err_str));
    }

    cmd_cp(&bak_file, XOBIN_PATH)?;

    println!("Successfully reverted the signature modification from backup");
    Ok(())
}

fn get_version() -> std::io::Result<String> {
    let conf_str = std::fs::read_to_string(RM_CONF)?;
    let re = Regex::new(r"REMARKABLE_RELEASE_VERSION=([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+)").unwrap();

    let Some(res) = re.captures(&conf_str) else {
        return Err(Error::new(
            ErrorKind::Other,
            format!("Can not find xochitl version"),
        ));
    };

    // safe to unwrap as None is caught above
    Ok(res.get(1).unwrap().as_str().to_owned())
}

// std::fs::copy will throw the error: Text file busy (os error 26)
// if xochitl is running, this will not
fn cmd_cp(from: &str, to: &str) -> std::io::Result<String> {
    let command_out = Command::new("/usr/bin/env")
        .args(["cp", from, to])
        .stdout(Stdio::piped())
        .output()?;
    String::from_utf8(command_out.stdout).map_err(|err| Error::new(ErrorKind::Other, err))
}

fn check_space() -> bool {
    let mut sys = System::new();
    sys.refresh_disks_list();

    let Some(root_disk_free_mb) = disk_free_mbyt(&sys, "/") else {
        println!("Can not find '/'");
        return false;
    };

    let Some(home_disk_free_mb) = disk_free_mbyt(&sys, "/home") else {
        println!("Can not find '/home'");
        return false;
    };

    let need = 1;
    if root_disk_free_mb < need {
        println!("Not enough space on '/'");
        println!("Have: {root_disk_free_mb}MB, Need: {need}MB");
        println!("Try to free space by running: journalctl --vacuum-time=1m");
        println!("Or: systemctl restart xochitl");
        return false;
    }
    let need = 12;
    if home_disk_free_mb < need {
        println!("Not enough space on '/home'");
        println!("Have: {root_disk_free_mb}MB, Need: {need}MB");
        return false;
    }

    true
}

fn disk_free_mbyt(sys: &System, disk: &str) -> Option<u64> {
    sys.disks()
        .into_iter()
        .filter(|v| v.mount_point() == Path::new(disk))
        .map(|v| v.available_space() / 1024 / 1024)
        .collect::<Vec<u64>>()
        .into_iter()
        .nth(0)
}
