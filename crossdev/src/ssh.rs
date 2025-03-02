use crate::{error::*, utils::*};
use std::{fs::File, io::Write, process::Command};
use xshell::Shell;

const OVERLAY_DIR: &str = "custom_buildroot/board/overlay";
const SSH_DIR: &str = "root/.ssh";
const SSH_KEY: &str = "id_rsa";
const SSH_BIN: &str = "ssh";
pub const SSH_PORT: &str = "10025";

pub fn prepare_ssh_key(download_dir: &str) -> Result<()> {
    let sh = Shell::new()?;

    let ssh_dir = format!("{download_dir}/{OVERLAY_DIR}/{SSH_DIR}");
    let ssh_key = format!("{ssh_dir}/{SSH_KEY}");
    let ssh_pub = format!("{ssh_key}.pub");
    let authorized_keys = format!("{ssh_dir}/authorized_keys");

    create_dir(&ssh_dir)?;

    if sh.path_exists(&ssh_key) && sh.path_exists(&authorized_keys) {
        println!("SSH key already exists");
        return Ok(());
    }

    // Generate SSH key without passphrase
    let status = Command::new("ssh-keygen")
        .arg("-t")
        .arg("rsa")
        .arg("-N")
        .arg("")
        .arg("-f")
        .arg(ssh_key.as_str())
        .status()?;

    if !status.success() {
        return Err(CrossDevError::SshKeyError);
    }

    // Create the authorized_keys file if it doesn't exist
    if !sh.path_exists(&authorized_keys) {
        File::create(&authorized_keys)?;
    }

    // Append the public key to the authorized_keys file
    let mut file = File::options().append(true).open(&authorized_keys)?;
    let pub_key = std::fs::read_to_string(&ssh_pub)?;
    writeln!(file, "{}", pub_key)?;

    println!("✅ SSH key is ready!");

    Ok(())
}

pub fn run_ssh(download_dir: &str) -> Result<()> {
    let ssh_dir = format!("{download_dir}/{OVERLAY_DIR}/{SSH_DIR}");
    let ssh_key = format!("{ssh_dir}/{SSH_KEY}");

    let ssh_args = [
        "-i",
        &ssh_key,
        "root@localhost",
        "-p",
        SSH_PORT,
        "-o",
        "StrictHostKeyChecking no",
    ];

    let mut child = Command::new(SSH_BIN)
        .args(ssh_args)
        .spawn()
        .expect("Failed to connect via ssh: {ssh_bin} {ssh_args:?}");

    let _ = child.wait().unwrap();

    Ok(())
}
