use std::ffi::CString;
use std::fs::Permissions;
use std::io;
use std::os::unix::prelude::{AsRawFd, PermissionsExt};

use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{close, execvp, fork, ForkResult};
use tempfile::NamedTempFile;

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let mut tmpfile = NamedTempFile::new()?;
    io::copy(&mut io::stdin().lock(), &mut tmpfile.as_file_mut())?;
    tmpfile
        .as_file()
        .set_permissions(Permissions::from_mode(0o100)) // --x------
        .expect("fialed to set permissions");
    close(tmpfile.as_file().as_raw_fd())?; // To avoid ETXTBSY in execvp

    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            let code = match waitpid(child, Some(WaitPidFlag::WUNTRACED))? {
                WaitStatus::Exited(_, code) => code,
                WaitStatus::Signaled(pid, signal, _) => {
                    eprintln!("{} received {}", pid, signal);
                    128 + signal as i32
                }
                WaitStatus::Stopped(pid, signal) => {
                    eprintln!("{} killed({})", pid, signal);
                    128 + signal as i32
                }
                _ => 127,
            };
            return Ok(code);
        }
        ForkResult::Child => {
            let path = match tmpfile.path().to_str() {
                Some(path) => CString::new(path)?,
                None => return Err("tmpfile path is not valid UTF-8".into()),
            };
            let args: Result<Vec<CString>, _> = std::env::args().map(CString::new).collect();
            let mut args = args?;
            assert!(!args.is_empty());
            args[0] = path;
            execvp(&args[0], &args)?;
        }
    }
    Ok(1)
}

fn main() {
    match run() {
        Ok(code) => {
            std::process::exit(code);
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
