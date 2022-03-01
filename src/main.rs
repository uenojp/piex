use std::ffi::CString;
use std::fs::Permissions;
use std::io;
use std::os::unix::prelude::{AsRawFd, PermissionsExt};

use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork};
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmpfile = NamedTempFile::new()?;
    io::copy(&mut io::stdin().lock(), &mut tmpfile.as_file_mut())?;
    tmpfile
        .as_file()
        .set_permissions(Permissions::from_mode(0o100)) // --x------
        .expect("fialed to set permissions");
    nix::unistd::close(tmpfile.as_file().as_raw_fd())?;

    match unsafe { fork() }? {
        nix::unistd::ForkResult::Parent { child } => {
            waitpid(child, None)?;
        }
        nix::unistd::ForkResult::Child => {
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
    Ok(())
}
