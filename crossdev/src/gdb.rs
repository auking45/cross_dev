use crate::{env::*, error::*, utils::*};
use std::{env, process::Command};

const GDB_DIR: &str = "gdb";
const GDB_BIN: &str = "gdb-multiarch";

pub fn run_gdb() -> Result<()> {
    let gdb_dir = format!("{}/{GDB_DIR}", get_dir(EnvType::RootDir)?);
    let work_dir = get_dir(EnvType::WorkDir)?;

    let _ = env::set_current_dir(&gdb_dir);

    let gdb_args = format!(
        r#"
        -x {gdb_dir}/.gdbinit
        --cd={gdb_dir}
        "#,
    );
    let gdb_args: Vec<_> = gdb_args.split_whitespace().collect();

    let mut child = Command::new(GDB_BIN)
        .args(gdb_args)
        .env("GDB_WORK_DIR", work_dir)
        .spawn()
        .expect("Failed to launch target: {gdb_bin} {gdb_args}");

    let _ = child.wait()?;

    Ok(())
}
