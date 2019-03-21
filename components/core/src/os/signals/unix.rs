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

//! Traps and notifies UNIX signals.

use std::{collections::VecDeque,
          sync::{Mutex,
                 Once,
                 ONCE_INIT}};

use crate::os::process::{Signal,
                         SignalCode};

use super::SignalEvent;

static INIT: Once = ONCE_INIT;

lazy_static::lazy_static! {
    static ref CAUGHT_SIGNALS: Mutex<VecDeque<SignalCode>> = Mutex::new(VecDeque::new());
}

// Functions from POSIX libc.
extern "C" {
    fn signal(sig: SignalCode,
              cb: unsafe extern "C" fn(SignalCode))
              -> unsafe extern "C" fn(SignalCode);
}

unsafe extern "C" fn handle_signal(signal: SignalCode) {
    CAUGHT_SIGNALS.lock()
                  .expect("Signal mutex poisoned")
                  .push_back(signal);
}

pub fn init() {
    INIT.call_once(|| {
            self::set_signal_handlers();
        });
}

/// Consumers should call this function fairly frequently and since the vast
/// majority of the time there is at most one signal event waiting, we return
/// at most one. If multiple signals have been received since the last call,
/// they will be returned, one per call in the order they were received.
pub fn check_for_signal() -> Option<SignalEvent> {
    let mut signals = CAUGHT_SIGNALS.lock().expect("Signal mutex poisoned");

    if let Some(code) = signals.pop_front() {
        match from_signal_code(code) {
            Some(Signal::INT) | Some(Signal::TERM) => Some(SignalEvent::Shutdown),
            Some(Signal::CHLD) => Some(SignalEvent::WaitForChild),
            Some(signal) => Some(SignalEvent::Passthrough(signal)),
            None => {
                println!("Received invalid signal: #{}", code);
                None
            }
        }
    } else {
        None
    }
}

fn set_signal_handlers() {
    unsafe {
        signal(libc::SIGHUP, handle_signal);
        signal(libc::SIGINT, handle_signal);
        signal(libc::SIGQUIT, handle_signal);
        signal(libc::SIGALRM, handle_signal);
        signal(libc::SIGTERM, handle_signal);
        signal(libc::SIGUSR1, handle_signal);
        signal(libc::SIGUSR2, handle_signal);
        signal(libc::SIGCHLD, handle_signal);
    }
}

/// These are the signals that we can eventually translate into
/// some kind of event
fn from_signal_code(code: SignalCode) -> Option<Signal> {
    // only 3 signals are explicitly acted upon... everything else
    // is passed through to the supervisor
    //
    // If it gets a HUP, it restarts the Supervisor
    // process... anything else will pretty much kill the
    // supervisor outright (I think)
    //
    // Also, these are things that get sent to the launcher.
    match code {
        libc::SIGHUP => Some(Signal::HUP),
        libc::SIGINT => Some(Signal::INT), // -> shutdown
        libc::SIGILL => Some(Signal::ILL),
        libc::SIGABRT => Some(Signal::ABRT),
        libc::SIGFPE => Some(Signal::FPE),
        libc::SIGKILL => Some(Signal::KILL),
        libc::SIGSEGV => Some(Signal::SEGV),
        libc::SIGTERM => Some(Signal::TERM), // -> wait on child
        libc::SIGCHLD => Some(Signal::CHLD), // -> shutdown
        _ => None,
    }
}
