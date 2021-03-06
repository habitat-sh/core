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

#[cfg(windows)]
pub mod windows_child;

#[allow(unused_variables)]
#[cfg(windows)]
mod windows;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
pub use self::windows::{become_command,
                        current_pid,
                        handle_from_pid,
                        is_alive,
                        Pid};

#[cfg(unix)]
pub(crate) use self::unix::SignalCode;
#[cfg(unix)]
pub use self::unix::{become_command,
                     current_pid,
                     is_alive,
                     signal,
                     Pid,
                     Signal};
