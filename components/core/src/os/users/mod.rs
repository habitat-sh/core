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

#[allow(unused_variables)]
#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::{assert_pkg_user_and_group,
                        can_run_services_as_svc_user,
                        get_current_groupname,
                        get_current_username,
                        get_effective_uid,
                        get_gid_by_name,
                        get_home_for_user,
                        get_uid_by_name,
                        root_level_account};

#[cfg(unix)]
pub mod linux;

#[cfg(unix)]
pub use self::linux::{assert_pkg_user_and_group,
                      can_run_services_as_svc_user,
                      get_current_groupname,
                      get_current_username,
                      get_effective_gid,
                      get_effective_groupname,
                      get_effective_uid,
                      get_effective_username,
                      get_gid_by_name,
                      get_home_for_user,
                      get_uid_by_name,
                      root_level_account};
