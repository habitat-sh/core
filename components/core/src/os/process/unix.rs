// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{ffi::OsString,
          io,
          os::unix::process::CommandExt,
          path::PathBuf,
          process::Command};

use libc::{self,
           pid_t};

use crate::error::{Error,
                   Result};

pub type Pid = libc::pid_t;
pub(crate) type SignalCode = libc::c_int;

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
pub enum Signal {
    INT,
    ILL,
    ABRT,
    FPE,
    KILL,
    SEGV,
    TERM,
    HUP,
    QUIT,
    ALRM,
    USR1,
    USR2,
    CHLD,
}

pub fn become_command(command: PathBuf, args: &[OsString]) -> Result<()> {
    become_exec_command(command, args)
}

/// Get process identifier of calling process.
pub fn current_pid() -> Pid { unsafe { libc::getpid() as pid_t } }

/// Determines if a process is running with the given process identifier.
pub fn is_alive(pid: Pid) -> bool {
    match unsafe { libc::kill(pid as pid_t, 0) } {
        0 => true,
        _ => {
            match io::Error::last_os_error().raw_os_error() {
                Some(libc::EPERM) => true,
                Some(libc::ESRCH) => false,
                _ => false,
            }
        }
    }
}

pub fn signal(pid: Pid, signal: Signal) -> Result<()> {
    unsafe {
        match libc::kill(pid as pid_t, signal.into()) {
            0 => Ok(()),
            e => Err(Error::SignalFailed(e, io::Error::last_os_error())),
        }
    }
}

impl From<Signal> for SignalCode {
    fn from(value: Signal) -> SignalCode {
        match value {
            Signal::INT => libc::SIGINT,
            Signal::ILL => libc::SIGILL,
            Signal::ABRT => libc::SIGABRT,
            Signal::FPE => libc::SIGFPE,
            Signal::KILL => libc::SIGKILL,
            Signal::SEGV => libc::SIGSEGV,
            Signal::TERM => libc::SIGTERM,
            Signal::HUP => libc::SIGHUP,
            Signal::QUIT => libc::SIGQUIT,
            Signal::ALRM => libc::SIGALRM,
            Signal::USR1 => libc::SIGUSR1,
            Signal::USR2 => libc::SIGUSR2,
            Signal::CHLD => libc::SIGCHLD,
        }
    }
}
/// Makes an `execvp(3)` system call to become a new program.
///
/// Note that if successful, this function will not return.
///
/// # Failures
///
/// * If the system call fails the error will be returned, otherwise this function does not return
fn become_exec_command(command: PathBuf, args: &[OsString]) -> Result<()> {
    debug!("Calling execvp(): ({:?}) {:?}", command.display(), &args);
    let error_if_failed = Command::new(command).args(args).exec();
    // The only possible return for the above function is an `Error` so return it, meaning that we
    // failed to exec to our target program
    Err(error_if_failed.into())
}
