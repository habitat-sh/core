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

//! Habitat package identifier representation.
//!

use std::borrow::Cow;
use std::cmp::{Ordering, PartialOrd};
use std::fmt;
use std::path::Path;
use std::result;
use std::str::FromStr;

use regex::Regex;
use serde;

use error::{Error, Result};
use package::PackageTarget;
use util;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Ident {
    Release(ReleaseIdent),
    Version(VersionIdent),
    Name(NameIdent),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReleaseIdent {
    origin: Origin,
    name: Name,
    version: Version,
    release: Release,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VersionIdent {
    origin: Origin,
    name: Name,
    version: Version,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NameIdent {
    origin: Origin,
    name: Name,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Origin(String);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Name(String);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version(String);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Release(String);

impl Ident {
    pub fn origin(&self) -> &Origin {
        match *self {
            Ident::Release(ref i) => i.origin(),
            Ident::Version(ref i) => i.origin(),
            Ident::Name(ref i) => i.origin(),
        }
    }

    pub fn name(&self) -> &Name {
        match *self {
            Ident::Release(ref i) => i.name(),
            Ident::Version(ref i) => i.name(),
            Ident::Name(ref i) => i.name(),
        }
    }

    pub fn version(&self) -> Option<&Version> {
        match *self {
            Ident::Release(ref i) => Some(i.version()),
            Ident::Version(ref i) => Some(i.version()),
            Ident::Name(_) => None,
        }
    }

    pub fn release(&self) -> Option<&Release> {
        match *self {
            Ident::Release(ref i) => Some(i.release()),
            Ident::Version(_) | Ident::Name(_) => None,
        }
    }

    pub fn iter(&self) -> IdentIter {
        IdentIter {
            inner: self,
            pos: 0,
        }
    }

    // TODO fn: This method scheduled for removal
    pub fn new<O, N, V, R>(
        origin: O,
        name: N,
        version: Option<V>,
        release: Option<R>,
    ) -> Result<Self>
    where
        O: Into<String>,
        N: Into<String>,
        V: Into<String>,
        R: Into<String>,
    {
        match (version, release) {
            (Some(version), Some(release)) => Ok(Ident::Release(ReleaseIdent::from_raw_parts(
                origin, name, version, release,
            )?)),
            (Some(version), None) => Ok(Ident::Version(VersionIdent::from_raw_parts(
                origin, name, version,
            )?)),
            (None, None) => Ok(Ident::Name(NameIdent::from_raw_parts(origin, name)?)),
            (None, Some(release)) => {
                return Err(Error::InvalidPackageIdent(format!(
                    "{}/{}//{}",
                    origin.into(),
                    name.into(),
                    release.into()
                )));
            }
        }
    }

    // TODO fn: move to RelaseIdent struct
    pub fn satisfies(&self, other: &Ident) -> bool {
        if self.origin() != other.origin() || self.name() != other.name() {
            return false;
        }
        if self.version().is_some() {
            if other.version().is_none() {
                return true;
            }
            if *self.version().unwrap() != *other.version().unwrap() {
                return false;
            }
        }
        if self.release().is_some() {
            if other.release().is_none() {
                return true;
            }
            if *self.release().unwrap() != *other.release().unwrap() {
                return false;
            }
        }
        true
    }

    // TODO fn: This method scheduled for removal
    pub fn fully_qualified(&self) -> bool {
        self.version().is_some() && self.release().is_some()
    }

    // TODO fn: This method scheduled for removal
    pub fn archive_name(&self) -> Result<String> {
        match *self {
            Ident::Release(ref i) => Ok(i.archive_name()),
            _ => Err(Error::FullyQualifiedPackageIdentRequired(self.to_string())),
        }
    }

    // TODO fn: This method scheduled for removal
    pub fn archive_name_with_target(&self, target: &PackageTarget) -> Result<String> {
        match *self {
            Ident::Release(ref i) => Ok(i.archive_name_with_target(target)),
            _ => Err(Error::FullyQualifiedPackageIdentRequired(self.to_string())),
        }
    }

    // TODO fn: This method scheduled for removal. In fact, we shouldn't have a "default" Ident. In
    // the meantime, there is some code which uses the `Default` impl heavily, so we're going to
    // use this function instead. Once we can update those call sites to an alternative that
    // doesn't involve defaults, this can go away. Hence the name. It's terribad.
    pub fn terribad_default() -> Self {
        Ident::Name(NameIdent::from_str("/").expect("Ident terribad default should parse"))
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ident::Release(ref i) => i.fmt(f),
            Ident::Version(ref i) => i.fmt(f),
            Ident::Name(ref i) => i.fmt(f),
        }
    }
}

impl FromStr for Ident {
    type Err = Error;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split("/").collect();
        match parts.len() {
            4 => Ok(Ident::Release(ReleaseIdent::from_raw_parts(
                parts[0], parts[1], parts[2], parts[3],
            )?)),
            3 => Ok(Ident::Version(VersionIdent::from_raw_parts(
                parts[0], parts[1], parts[2],
            )?)),
            2 => Ok(Ident::Name(NameIdent::from_raw_parts(parts[0], parts[1])?)),
            _ => return Err(Error::InvalidPackageIdent(value.to_string())),
        }
    }
}

impl serde::Serialize for Ident {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'d> serde::Deserialize<'d> for Ident {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        util::deserialize_using_from_str(deserializer)
    }
}

impl AsRef<Ident> for Ident {
    fn as_ref(&self) -> &Ident {
        self
    }
}

impl PartialOrd for Ident {
    /// Packages can be compared according to the following:
    ///
    /// * origin is ignored in the comparison - my redis and
    ///   your redis compare the same.
    /// * If the names are not equal, they cannot be compared.
    /// * If the versions are greater/lesser, return that as
    ///   the ordering.
    /// * If the versions are equal, return the greater/lesser
    ///   for the release.
    fn partial_cmp(&self, other: &Ident) -> Option<Ordering> {
        if self.name() != other.name() {
            return None;
        }

        match (self, other) {
            // * If the versions are both missing, we cannot compare
            // * if both releases are missing, we cannot compare
            (Ident::Name(_), Ident::Name(_)) | (Ident::Version(_), Ident::Version(_)) => None,
            // * If my version is missing and the other has a version, return `Less`
            // * If my release is missing and the other has a release, return `Less`
            (Ident::Name(_), Ident::Version(_))
            | (Ident::Name(_), Ident::Release(_))
            | (Ident::Version(_), Ident::Release(_)) => Some(Ordering::Less),
            // * If I have a version and the other is missing a version, return `Greater`
            // * If I have a release and the other is missing a release, return `Greater`
            (Ident::Version(_), Ident::Name(_))
            | (Ident::Release(_), Ident::Name(_))
            | (Ident::Release(_), Ident::Version(_)) => Some(Ordering::Greater),
            // * If I have a release and the other has a release, then sort the two
            (Ident::Release(self_r), Ident::Release(other_r)) => self_r.partial_cmp(other_r),
        }
    }
}

impl Ord for Ident {
    /// Packages can be compared according to the following:
    ///
    /// * origin is ignored in the comparison - my redis and
    ///   your redis compare the same.
    /// * If the names are not equal, they cannot be compared.
    /// * If the versions are greater/lesser, return that as
    ///   the ordering.
    /// * If the versions are equal, return the greater/lesser
    ///   for the release.
    fn cmp(&self, other: &Ident) -> Ordering {
        if self.name() != other.name() {
            return self.name().cmp(&other.name());
        }
        // TODO fn: probably needs refactoring/rework. I'm nervous seeing these unconditional
        // unwraps for version
        match pkg_version_sort(self.version().unwrap(), other.version().unwrap()) {
            ord @ Ok(Ordering::Greater) | ord @ Ok(Ordering::Less) => ord.unwrap(),
            Ok(Ordering::Equal) => self.release().cmp(&other.release()),
            Err(_) => Ordering::Less,
        }
    }
}

// TODO fn: possibly not needed anymore? initially created for `FullyQualifiedPackageIdent` in
// `common::command::install` module
impl<'a> From<Ident> for Cow<'a, Ident> {
    fn from(ident: Ident) -> Cow<'a, Ident> {
        Cow::Owned(ident)
    }
}

// TODO fn: possibly not needed anymore? initially created for `FullyQualifiedPackageIdent` in
// `common::command::install` module
impl<'a> From<&'a Ident> for Cow<'a, Ident> {
    fn from(ident: &'a Ident) -> Cow<'a, Ident> {
        Cow::Borrowed(ident)
    }
}

impl ReleaseIdent {
    pub fn new<O, N, V, R>(origin: O, name: N, version: V, release: R) -> Self
    where
        O: Into<Origin>,
        N: Into<Name>,
        V: Into<Version>,
        R: Into<Release>,
    {
        ReleaseIdent {
            origin: origin.into(),
            name: name.into(),
            version: version.into(),
            release: release.into(),
        }
    }

    pub fn from_raw_parts<O, N, V, R>(origin: O, name: N, version: V, release: R) -> Result<Self>
    where
        O: Into<String>,
        N: Into<String>,
        V: Into<String>,
        R: Into<String>,
    {
        Ok(Self::new(
            Origin::new(origin)?,
            Name::new(name)?,
            Version::new(version)?,
            Release::new(release)?,
        ))
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn release(&self) -> &Release {
        &self.release
    }

    pub fn iter(&self) -> ReleaseIdentIter {
        ReleaseIdentIter {
            inner: self,
            pos: 0,
        }
    }

    pub fn archive_name(&self) -> String {
        self.archive_name_with_target(PackageTarget::active_target())
    }

    pub fn archive_name_with_target(&self, target: &PackageTarget) -> String {
        format!(
            "{}-{}-{}-{}-{}.hart",
            self.origin, self.name, self.version, self.release, target
        )
    }
}

impl fmt::Display for ReleaseIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.origin, self.name, self.version, self.release
        )
    }
}

impl FromStr for ReleaseIdent {
    type Err = Error;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split("/").collect();
        match parts.len() {
            4 => ReleaseIdent::from_raw_parts(parts[0], parts[1], parts[2], parts[3]),
            _ => return Err(Error::InvalidReleaseIdent(value.to_string())),
        }
    }
}

impl serde::Serialize for ReleaseIdent {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'d> serde::Deserialize<'d> for ReleaseIdent {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        util::deserialize_using_from_str(deserializer)
    }
}

impl From<ReleaseIdent> for Ident {
    fn from(ident: ReleaseIdent) -> Self {
        Ident::Release(ident)
    }
}

impl PartialOrd for ReleaseIdent {
    fn partial_cmp(&self, other: &ReleaseIdent) -> Option<Ordering> {
        match pkg_version_sort(self.version(), other.version()) {
            ord @ Ok(Ordering::Greater) | ord @ Ok(Ordering::Less) => ord.ok(),
            Ok(Ordering::Equal) => Some(self.release().cmp(other.release())),
            Err(_) => {
                // TODO SA: Can we do better than this? As long as we allow
                // non-numeric versions to co-exist with numeric ones, we
                // always have potential for incorrect ordering no matter
                // what we choose - eg, "master" vs. "0.x.x" (real examples)
                debug!(
                    "Comparing non-numeric versions: {} {}",
                    self.version(),
                    other.version()
                );
                match self.version().cmp(other.version()) {
                    ord @ Ordering::Greater | ord @ Ordering::Less => Some(ord),
                    Ordering::Equal => Some(self.release().cmp(other.release())),
                }
            }
        }
    }
}

impl VersionIdent {
    pub fn new<O, N, V>(origin: O, name: N, version: V) -> Self
    where
        O: Into<Origin>,
        N: Into<Name>,
        V: Into<Version>,
    {
        VersionIdent {
            origin: origin.into(),
            name: name.into(),
            version: version.into(),
        }
    }

    pub fn from_raw_parts<O, N, V>(origin: O, name: N, version: V) -> Result<Self>
    where
        O: Into<String>,
        N: Into<String>,
        V: Into<String>,
    {
        Ok(Self::new(
            Origin::new(origin)?,
            Name::new(name)?,
            Version::new(version)?,
        ))
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn iter(&self) -> VersionIdentIter {
        VersionIdentIter {
            inner: self,
            pos: 0,
        }
    }
}

impl fmt::Display for VersionIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}/{}", self.origin, self.name, self.version)
    }
}

impl FromStr for VersionIdent {
    type Err = Error;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split("/").collect();
        match parts.len() {
            3 => VersionIdent::from_raw_parts(parts[0], parts[1], parts[2]),
            _ => return Err(Error::InvalidVersionIdent(value.to_string())),
        }
    }
}

impl serde::Serialize for VersionIdent {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'d> serde::Deserialize<'d> for VersionIdent {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        util::deserialize_using_from_str(deserializer)
    }
}

impl From<VersionIdent> for Ident {
    fn from(ident: VersionIdent) -> Self {
        Ident::Version(ident)
    }
}

impl NameIdent {
    pub fn new<O, N>(origin: O, name: N) -> Self
    where
        O: Into<Origin>,
        N: Into<Name>,
    {
        NameIdent {
            origin: origin.into(),
            name: name.into(),
        }
    }

    pub fn from_raw_parts<O, N>(origin: O, name: N) -> Result<Self>
    where
        O: Into<String>,
        N: Into<String>,
    {
        Ok(Self::new(Origin::new(origin)?, Name::new(name)?))
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn iter(&self) -> NameIdentIter {
        NameIdentIter {
            inner: self,
            pos: 0,
        }
    }
}

impl fmt::Display for NameIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.origin, self.name)
    }
}

impl FromStr for NameIdent {
    type Err = Error;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split("/").collect();
        match parts.len() {
            2 => NameIdent::from_raw_parts(parts[0], parts[1]),
            _ => return Err(Error::InvalidNameIdent(value.to_string())),
        }
    }
}

impl serde::Serialize for NameIdent {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'d> serde::Deserialize<'d> for NameIdent {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        util::deserialize_using_from_str(deserializer)
    }
}

impl From<NameIdent> for Ident {
    fn from(ident: NameIdent) -> Self {
        Ident::Name(ident)
    }
}

impl Origin {
    pub fn new<S: Into<String>>(origin: S) -> Result<Self> {
        Ok(Origin(origin.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

impl FromStr for Origin {
    type Err = Error;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        Origin::new(value)
    }
}

impl<'a> From<&'a Origin> for Origin {
    fn from(p: &'a Origin) -> Self {
        p.to_owned()
    }
}

impl<'a> From<&'a Origin> for String {
    fn from(p: &'a Origin) -> Self {
        p.0.clone()
    }
}

impl<'a> From<&'a Origin> for &'a str {
    fn from(p: &'a Origin) -> Self {
        p.0.as_str()
    }
}

impl AsRef<str> for Origin {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for Origin {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl Name {
    pub fn new<S: Into<String>>(name: S) -> Result<Self> {
        Ok(Name(name.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

impl FromStr for Name {
    type Err = Error;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        Name::new(value)
    }
}

impl<'a> From<&'a Name> for Name {
    fn from(p: &'a Name) -> Self {
        p.to_owned()
    }
}

impl<'a> From<&'a Name> for String {
    fn from(p: &'a Name) -> Self {
        p.0.clone()
    }
}

impl<'a> From<&'a Name> for &'a str {
    fn from(p: &'a Name) -> Self {
        p.0.as_str()
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for Name {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl Version {
    pub fn new<S: Into<String>>(version: S) -> Result<Self> {
        Ok(Version(version.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        Version::new(value)
    }
}

impl<'a> From<&'a Version> for Version {
    fn from(p: &'a Version) -> Self {
        p.to_owned()
    }
}

impl<'a> From<&'a Version> for String {
    fn from(p: &'a Version) -> Self {
        p.0.clone()
    }
}

impl<'a> From<&'a Version> for &'a str {
    fn from(p: &'a Version) -> Self {
        p.0.as_str()
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for Version {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl Release {
    pub fn new<S: Into<String>>(release: S) -> Result<Self> {
        Ok(Release(release.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

impl FromStr for Release {
    type Err = Error;

    fn from_str(value: &str) -> result::Result<Self, Self::Err> {
        Release::new(value)
    }
}

impl<'a> From<&'a Release> for Release {
    fn from(p: &'a Release) -> Self {
        p.to_owned()
    }
}

impl<'a> From<&'a Release> for String {
    fn from(p: &'a Release) -> Self {
        p.0.clone()
    }
}

impl<'a> From<&'a Release> for &'a str {
    fn from(p: &'a Release) -> Self {
        p.0.as_str()
    }
}

impl AsRef<Path> for Release {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl AsRef<str> for Release {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

pub struct IdentIter<'a> {
    inner: &'a Ident,
    pos: usize,
}

impl<'a> Iterator for IdentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        match *self.inner {
            Ident::Release(ref i) => i.iter().item_for_pos(self.pos),
            Ident::Version(ref i) => i.iter().item_for_pos(self.pos),
            Ident::Name(ref i) => i.iter().item_for_pos(self.pos),
        }
    }
}

pub struct ReleaseIdentIter<'a> {
    inner: &'a ReleaseIdent,
    pos: usize,
}

impl<'a> ReleaseIdentIter<'a> {
    fn item_for_pos(&self, pos: usize) -> Option<&'a str> {
        match pos {
            1 => Some(self.inner.origin().as_str()),
            2 => Some(self.inner.name().as_str()),
            3 => Some(self.inner.version().as_str()),
            4 => Some(self.inner.release().as_str()),
            _ => None,
        }
    }
}

impl<'a> Iterator for ReleaseIdentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        self.item_for_pos(self.pos)
    }
}

pub struct VersionIdentIter<'a> {
    inner: &'a VersionIdent,
    pos: usize,
}

impl<'a> VersionIdentIter<'a> {
    fn item_for_pos(&self, pos: usize) -> Option<&'a str> {
        match pos {
            1 => Some(self.inner.origin().as_str()),
            2 => Some(self.inner.name().as_str()),
            3 => Some(self.inner.version().as_str()),
            _ => None,
        }
    }
}

impl<'a> Iterator for VersionIdentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        self.item_for_pos(self.pos)
    }
}

pub struct NameIdentIter<'a> {
    inner: &'a NameIdent,
    pos: usize,
}

impl<'a> NameIdentIter<'a> {
    fn item_for_pos(&self, pos: usize) -> Option<&'a str> {
        match pos {
            1 => Some(self.inner.origin().as_str()),
            2 => Some(self.inner.name().as_str()),
            _ => None,
        }
    }
}

impl<'a> Iterator for NameIdentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        self.item_for_pos(self.pos)
    }
}

// TODO fn: this trait is scheduled for removal
pub trait Identifiable: fmt::Display + Into<Ident> {
    fn origin(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> Option<&str>;
    fn release(&self) -> Option<&str>;

    fn fully_qualified(&self) -> bool {
        self.version().is_some() && self.release().is_some()
    }
}

// TODO fn: this impl is scheduled for removal
impl Identifiable for Ident {
    fn origin(&self) -> &str {
        self.origin().as_str()
    }

    fn name(&self) -> &str {
        self.name().as_str()
    }

    fn version(&self) -> Option<&str> {
        self.version().map(|f| f.as_str())
    }

    fn release(&self) -> Option<&str> {
        self.release().map(|f| f.as_str())
    }
}

lazy_static! {
    static ref ORIGIN_NAME_RE: Regex =
        Regex::new(r"\A[a-z0-9][a-z0-9_-]*\z").expect("Unable to compile regex");
}

// TODO fn: remove shim an update params of `version_sort()`
fn pkg_version_sort(a_version: &Version, b_version: &Version) -> Result<Ordering> {
    version_sort(a_version.as_str(), b_version.as_str())
}

/// Sorts two packages according to their version.
///
/// We are a bit more strict than your average package management solution on versioning.
/// What we support is the "some number of digits or dots" (the version number),
/// followed by an optional "-" and any alphanumeric string (the extension). When determining sort
/// order, we:
///
/// * Separate the version numbers from the extensions
/// * Split the version numbers into an array of digits on any '.' characters. Digits are converted
///   into <u64>.
/// * Compare the version numbers by iterating over them. If 'a' is greater or lesser than 'b', we
///   return that as the result. If it is equal, we move to the next digit and repeat. If one of
///   the version numbers is exhausted before the other, it gains 0's for the missing slot.
/// * If the version numbers are equal, but either A or B has an extension (but not both) than the
///   version without the extension is greater. (1.0.0 is greater than 1.0.0-alpha6)
/// * If both have an extension, it is compared lexicographically, with the result as the final
///   ordering.
///
/// Returns a Error if we fail to match for any reason.
// TODO fn: does this need to be public API?
pub fn version_sort(a_version: &str, b_version: &str) -> Result<Ordering> {
    let (a_parts, a_extension) = split_version(a_version)?;
    let (b_parts, b_extension) = split_version(b_version)?;
    let mut a_iter = a_parts.iter();
    let mut b_iter = b_parts.iter();
    loop {
        let mut a_exhausted = false;
        let mut b_exhausted = false;
        let a_num = match a_iter.next() {
            Some(i) => i.parse::<u64>()?,
            None => {
                a_exhausted = true;
                0u64
            }
        };
        let b_num = match b_iter.next() {
            Some(i) => i.parse::<u64>()?,
            None => {
                b_exhausted = true;
                0u64
            }
        };
        if a_exhausted && b_exhausted {
            break;
        }
        match a_num.cmp(&b_num) {
            Ordering::Greater => {
                return Ok(Ordering::Greater);
            }
            Ordering::Equal => {
                continue;
            }
            Ordering::Less => {
                return Ok(Ordering::Less);
            }
        }
    }

    // If you have equal digits, and one has an extension, it is
    // the plain digits who win.
    // 1.0.0-alpha1 vs 1.0.0
    if a_extension.is_some() && b_extension.is_none() {
        return Ok(Ordering::Less);
    } else if a_extension.is_none() && b_extension.is_some() {
        return Ok(Ordering::Greater);
    } else if a_extension.is_none() && b_extension.is_none() {
        return Ok(Ordering::Equal);
    } else {
        let a = match a_extension {
            Some(a) => a,
            None => String::new(),
        };
        let b = match b_extension {
            Some(b) => b,
            None => String::new(),
        };
        return Ok(a.cmp(&b));
    }
}

fn split_version(version: &str) -> Result<(Vec<&str>, Option<String>)> {
    let re = Regex::new(r"([\d\.]+)(.+)?")?;
    let caps = match re.captures(version) {
        Some(caps) => caps,
        None => return Err(Error::InvalidPackageIdent(version.to_string())),
    };
    let version_number = caps.get(1).unwrap();
    let extension = match caps.get(2) {
        Some(e) => {
            let mut estr: String = e.as_str().to_string();
            if estr.len() > 1 && estr.chars().nth(0).unwrap() == '-' {
                estr.remove(0);
            }
            Some(estr)
        }
        None => None,
    };
    let version_parts: Vec<&str> = version_number.as_str().split('.').collect();
    Ok((version_parts, extension))
}

/// Is the string a valid origin name?
pub fn is_valid_origin_name(origin: &str) -> bool {
    origin.chars().count() <= 255 && ORIGIN_NAME_RE.is_match(origin)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensures that this "terribad" default will not unwrap or panic given any future validation of
    // origin or name components of an ident.
    #[test]
    fn terribad_default() {
        Ident::terribad_default();
    }

    mod release_ident {
        use super::*;

        use std::path::PathBuf;

        use package::target;

        use toml;

        fn ident(s: &str) -> ReleaseIdent {
            ReleaseIdent::from_str(s).unwrap()
        }

        #[test]
        fn new() {
            let origin = Origin::new("chromeo").unwrap();
            let name = Name::new("room-service").unwrap();
            let version = Version::new("1.0.1").unwrap();
            let release = Release::new("20180810134905").unwrap();

            // The only reason we're cloning here is to have another copy for the assertions below
            // as a testing convenience.  This constructor takes ownership of its parameters by
            // design.
            let ident = ReleaseIdent::new(
                origin.clone(),
                name.clone(),
                version.clone(),
                release.clone(),
            );

            assert_eq!(&origin, ident.origin());
            assert_eq!(&name, ident.name());
            assert_eq!(&version, ident.version());
            assert_eq!(&release, ident.release());
        }

        #[test]
        fn from_raw_parts() {
            let ident = ReleaseIdent::from_raw_parts(
                "neal-morse-band",
                "long-day",
                "9.0.9",
                "20180810140105",
            )
            .unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("long-day").unwrap(), ident.name());
            assert_eq!(&Version::new("9.0.9").unwrap(), ident.version());
            assert_eq!(&Release::new("20180810140105").unwrap(), ident.release());
        }

        #[test]
        fn from_raw_parts_mixed_params() {
            let ident = ReleaseIdent::from_raw_parts(
                // a `&str`
                "neal-morse-band",
                // an owned `String
                String::from("long-day"),
                // a `Cow` from a `Path`
                Path::new("9.0.9").to_string_lossy(),
                // a `Cow` from a `PathBuf`
                PathBuf::from("20180810140105").to_string_lossy(),
            )
            .unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("long-day").unwrap(), ident.name());
            assert_eq!(&Version::new("9.0.9").unwrap(), ident.version());
            assert_eq!(&Release::new("20180810140105").unwrap(), ident.release());
        }

        // TODO fn: add `raw_from_parts` testing when validation is introduced

        #[test]
        fn iter() {
            let ident = ident("neal-morse-band/slave-to-your-mind/2.0.1/20180810145506");
            let mut iter = ident.iter();

            assert_eq!(Some("neal-morse-band"), iter.next());
            assert_eq!(Some("slave-to-your-mind"), iter.next());
            assert_eq!(Some("2.0.1"), iter.next());
            assert_eq!(Some("20180810145506"), iter.next());
        }

        #[test]
        fn to_string() {
            let ident = ident("neal-morse-band/long-day/9.0.9/20180810140105");

            assert_eq!(
                String::from("neal-morse-band/long-day/9.0.9/20180810140105"),
                ident.to_string()
            );
        }

        #[test]
        fn from_str() {
            let ident =
                ReleaseIdent::from_str("neal-morse-band/makes-no-sense/3.2.1/20180810140105")
                    .unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("makes-no-sense").unwrap(), ident.name());
            assert_eq!(&Version::new("3.2.1").unwrap(), ident.version());
            assert_eq!(&Release::new("20180810140105").unwrap(), ident.release());
        }

        #[test]
        fn from_str_missing_release_part() {
            let s = "neal-morse-band/makes-no-sense/3.2.1";

            match ReleaseIdent::from_str(s) {
                Err(Error::InvalidReleaseIdent(ref val)) => assert_eq!(val, s),
                Err(e) => panic!("ReleaseIdent::from_str failed with wrong error type: {}", e),
                Ok(_) => panic!("ReleaseIdent::from_str should fail to parse: {}", s),
            }
        }

        #[test]
        fn from_str_missing_version_part() {
            let s = "neal-morse-band/makes-no-sense";

            match ReleaseIdent::from_str(s) {
                Err(Error::InvalidReleaseIdent(ref val)) => assert_eq!(val, s),
                Err(e) => panic!("ReleaseIdent::from_str failed with wrong error type: {}", e),
                Ok(_) => panic!("ReleaseIdent::from_str should fail to parse: {}", s),
            }
        }

        // TODO fn: add `from_str` testing when validation is introduced

        // Sanity test for `String`-to-`String` round tripping
        #[test]
        fn from_str_to_string_round_trip() {
            let expected = String::from("neal-morse-band/makes-no-sense/3.2.1/20180810140105");

            assert_eq!(
                expected,
                ReleaseIdent::from_str(&expected).unwrap().to_string()
            );
        }

        #[test]
        fn serialize() {
            #[derive(Serialize)]
            struct Data {
                ident: ReleaseIdent,
            }
            let data = Data {
                ident: ident("neal-morse-band/makes-no-sense/3.2.1/20180810140105"),
            };
            let toml = toml::to_string(&data).unwrap();

            assert!(toml
                .starts_with(r#"ident = "neal-morse-band/makes-no-sense/3.2.1/20180810140105""#));
        }

        #[test]
        fn deserialize() {
            #[derive(Deserialize)]
            struct Data {
                ident: ReleaseIdent,
            }
            let toml = r#"
            ident = "neal-morse-band/makes-no-sense/3.2.1/20180810140105"
            "#;
            let data: Data = toml::from_str(toml).unwrap();

            assert_eq!(
                data.ident,
                ident("neal-morse-band/makes-no-sense/3.2.1/20180810140105"),
            );
        }

        // Sanity test for Serialize/Deserialize round tripping
        #[test]
        fn serialize_deserialize_round_trip() {
            #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
            struct Data {
                ident: ReleaseIdent,
            }
            let expected = Data {
                ident: ident("neal-morse-band/makes-no-sense/3.2.1/20180810140105"),
            };

            assert_eq!(
                expected,
                toml::from_str(&toml::to_string(&expected).unwrap()).unwrap()
            );
        }

        #[test]
        fn into_ident() {
            let ident = ident("neal_morse_band/slave-to-your-mind/2.0.1/20180810145506");

            assert_eq!(Ident::Release(ident.clone()), ident.into());
        }

        // TODO fn: test PartialOrd impl

        #[test]
        fn archive_name() {
            let ident = ident("neal_morse_band/slave-to-your-mind/2.0.1/20180810145506");
            let expected = format!(
                "neal_morse_band-slave-to-your-mind-2.0.1-20180810145506-{}.hart",
                PackageTarget::active_target()
            );

            assert_eq!(expected, ident.archive_name());
        }

        #[test]
        fn archive_name_with_target() {
            let ident = ident("neal_morse_band/slave-to-your-mind/2.0.1/20180810145506");
            let expected = format!(
                "neal_morse_band-slave-to-your-mind-2.0.1-20180810145506-{}.hart",
                target::X86_64_DARWIN
            );

            assert_eq!(
                expected,
                ident.archive_name_with_target(&target::X86_64_DARWIN),
            );
        }
    }

    mod version_ident {
        use super::*;

        use toml;

        fn ident(s: &str) -> VersionIdent {
            VersionIdent::from_str(s).unwrap()
        }

        #[test]
        fn new() {
            let origin = Origin::new("chromeo").unwrap();
            let name = Name::new("room-service").unwrap();
            let version = Version::new("1.0.1").unwrap();

            // The only reason we're cloning here is to have another copy for the assertions below
            // as a testing convenience.  This constructor takes ownership of its parameters by
            // design.
            let ident = VersionIdent::new(origin.clone(), name.clone(), version.clone());

            assert_eq!(&origin, ident.origin());
            assert_eq!(&name, ident.name());
            assert_eq!(&version, ident.version());
        }

        #[test]
        fn from_raw_parts() {
            let ident =
                VersionIdent::from_raw_parts("neal-morse-band", "long-day", "9.0.9").unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("long-day").unwrap(), ident.name());
            assert_eq!(&Version::new("9.0.9").unwrap(), ident.version());
        }

        #[test]
        fn from_raw_parts_mixed_params() {
            let ident = VersionIdent::from_raw_parts(
                // a `&str`
                "neal-morse-band",
                // an owned `String
                String::from("long-day"),
                // a `Cow` from a `Path`
                Path::new("9.0.9").to_string_lossy(),
            )
            .unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("long-day").unwrap(), ident.name());
            assert_eq!(&Version::new("9.0.9").unwrap(), ident.version());
        }

        // TODO fn: add `raw_from_parts` testing when validation is introduced

        #[test]
        fn iter() {
            let ident = ident("neal-morse-band/slave-to-your-mind/2.0.1");
            let mut iter = ident.iter();

            assert_eq!(Some("neal-morse-band"), iter.next());
            assert_eq!(Some("slave-to-your-mind"), iter.next());
            assert_eq!(Some("2.0.1"), iter.next());
        }

        #[test]
        fn to_string() {
            let ident = ident("neal-morse-band/long-day/9.0.9");

            assert_eq!(
                String::from("neal-morse-band/long-day/9.0.9"),
                ident.to_string()
            );
        }

        #[test]
        fn from_str() {
            let ident = VersionIdent::from_str("neal-morse-band/makes-no-sense/3.2.1").unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("makes-no-sense").unwrap(), ident.name());
            assert_eq!(&Version::new("3.2.1").unwrap(), ident.version());
        }

        #[test]
        fn from_str_including_release_part() {
            let s = "neal-morse-band/makes-no-sense/3.2.1/20180810151301";

            match VersionIdent::from_str(s) {
                Err(Error::InvalidVersionIdent(ref val)) => assert_eq!(val, s),
                Err(e) => panic!("VersionIdent::from_str failed with wrong error type: {}", e),
                Ok(_) => panic!("VersionIdent::from_str should fail to parse: {}", s),
            }
        }

        #[test]
        fn from_str_missing_version_part() {
            let s = "neal-morse-band/makes-no-sense";

            match VersionIdent::from_str(s) {
                Err(Error::InvalidVersionIdent(ref val)) => assert_eq!(val, s),
                Err(e) => panic!("VersionIdent::from_str failed with wrong error type: {}", e),
                Ok(_) => panic!("VersionIdent::from_str should fail to parse: {}", s),
            }
        }

        // TODO fn: add `from_str` testing when validation is introduced

        // Sanity test for `String`-to-`String` round tripping
        #[test]
        fn from_str_to_string_round_trip() {
            let expected = String::from("neal-morse-band/makes-no-sense/3.2.1");

            assert_eq!(
                expected,
                VersionIdent::from_str(&expected).unwrap().to_string()
            );
        }

        #[test]
        fn serialize() {
            #[derive(Serialize)]
            struct Data {
                ident: VersionIdent,
            }
            let data = Data {
                ident: ident("neal-morse-band/makes-no-sense/3.2.1"),
            };
            let toml = toml::to_string(&data).unwrap();

            assert!(toml.starts_with(r#"ident = "neal-morse-band/makes-no-sense/3.2.1""#));
        }

        #[test]
        fn deserialize() {
            #[derive(Deserialize)]
            struct Data {
                ident: VersionIdent,
            }
            let toml = r#"
            ident = "neal-morse-band/makes-no-sense/3.2.1"
            "#;
            let data: Data = toml::from_str(toml).unwrap();

            assert_eq!(data.ident, ident("neal-morse-band/makes-no-sense/3.2.1"),);
        }

        // Sanity test for Serialize/Deserialize round tripping
        #[test]
        fn serialize_deserialize_round_trip() {
            #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
            struct Data {
                ident: VersionIdent,
            }
            let expected = Data {
                ident: ident("neal-morse-band/makes-no-sense/3.2.1"),
            };

            assert_eq!(
                expected,
                toml::from_str(&toml::to_string(&expected).unwrap()).unwrap()
            );
        }

        #[test]
        fn into_ident() {
            let ident = ident("neal_morse_band/slave-to-your-mind/2.0.1");

            assert_eq!(Ident::Version(ident.clone()), ident.into());
        }
    }

    mod name_ident {
        use super::*;

        use toml;

        fn ident(s: &str) -> NameIdent {
            NameIdent::from_str(s).unwrap()
        }

        #[test]
        fn new() {
            let origin = Origin::new("chromeo").unwrap();
            let name = Name::new("room-service").unwrap();

            // The only reason we're cloning here is to have another copy for the assertions below
            // as a testing convenience.  This constructor takes ownership of its parameters by
            // design.
            let ident = NameIdent::new(origin.clone(), name.clone());

            assert_eq!(&origin, ident.origin());
            assert_eq!(&name, ident.name());
        }

        #[test]
        fn from_raw_parts() {
            let ident = NameIdent::from_raw_parts("neal-morse-band", "long-day").unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("long-day").unwrap(), ident.name());
        }

        #[test]
        fn from_raw_parts_mixed_params() {
            let ident = NameIdent::from_raw_parts(
                // a `&str`
                "neal-morse-band",
                // an owned `String
                String::from("long-day"),
            )
            .unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("long-day").unwrap(), ident.name());
        }

        // TODO fn: add `raw_from_parts` testing when validation is introduced

        #[test]
        fn iter() {
            let ident = ident("neal-morse-band/slave-to-your-mind");
            let mut iter = ident.iter();

            assert_eq!(Some("neal-morse-band"), iter.next());
            assert_eq!(Some("slave-to-your-mind"), iter.next());
        }

        #[test]
        fn to_string() {
            let ident = ident("neal-morse-band/long-day");

            assert_eq!(String::from("neal-morse-band/long-day"), ident.to_string());
        }

        #[test]
        fn from_str() {
            let ident = NameIdent::from_str("neal-morse-band/makes-no-sense").unwrap();

            assert_eq!(&Origin::new("neal-morse-band").unwrap(), ident.origin());
            assert_eq!(&Name::new("makes-no-sense").unwrap(), ident.name());
        }

        #[test]
        fn from_str_including_release_part() {
            let s = "neal-morse-band/makes-no-sense/3.2.1/20180810151301";

            match NameIdent::from_str(s) {
                Err(Error::InvalidNameIdent(ref val)) => assert_eq!(val, s),
                Err(e) => panic!("NameIdent::from_str failed with wrong error type: {}", e),
                Ok(_) => panic!("NameIdent::from_str should fail to parse: {}", s),
            }
        }

        #[test]
        fn from_str_including_version_part() {
            let s = "neal-morse-band/makes-no-sense/3.2.1";

            match NameIdent::from_str(s) {
                Err(Error::InvalidNameIdent(ref val)) => assert_eq!(val, s),
                Err(e) => panic!("NameIdent::from_str failed with wrong error type: {}", e),
                Ok(_) => panic!("NameIdent::from_str should fail to parse: {}", s),
            }
        }

        // TODO fn: add `from_str` testing when validation is introduced

        // Sanity test for `String`-to-`String` round tripping
        #[test]
        fn from_str_to_string_round_trip() {
            let expected = String::from("neal-morse-band/makes-no-sense");

            assert_eq!(
                expected,
                NameIdent::from_str(&expected).unwrap().to_string()
            );
        }

        #[test]
        fn serialize() {
            #[derive(Serialize)]
            struct Data {
                ident: NameIdent,
            }
            let data = Data {
                ident: ident("neal-morse-band/makes-no-sense"),
            };
            let toml = toml::to_string(&data).unwrap();

            assert!(toml.starts_with(r#"ident = "neal-morse-band/makes-no-sense""#));
        }

        #[test]
        fn deserialize() {
            #[derive(Deserialize)]
            struct Data {
                ident: NameIdent,
            }
            let toml = r#"
            ident = "neal-morse-band/makes-no-sense"
            "#;
            let data: Data = toml::from_str(toml).unwrap();

            assert_eq!(data.ident, ident("neal-morse-band/makes-no-sense"),);
        }

        // Sanity test for Serialize/Deserialize round tripping
        #[test]
        fn serialize_deserialize_round_trip() {
            #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
            struct Data {
                ident: NameIdent,
            }
            let expected = Data {
                ident: ident("neal-morse-band/makes-no-sense"),
            };

            assert_eq!(
                expected,
                toml::from_str(&toml::to_string(&expected).unwrap()).unwrap()
            );
        }

        #[test]
        fn into_ident() {
            let ident = ident("neal_morse_band/slave-to-your-mind");

            assert_eq!(Ident::Name(ident.clone()), ident.into());
        }
    }

    mod ident {
        use super::*;

        // TODO fn: This is scheduled for removal
        #[test]
        fn new_for_release() {
            let ident = Ident::new(
                "neal-morse-band",
                "long-day",
                Some("9.0.9"),
                Some("20180810140105"),
            )
            .unwrap();

            assert_eq!(
                Ident::Release(
                    ReleaseIdent::from_str("neal-morse-band/long-day/9.0.9/20180810140105")
                        .unwrap()
                ),
                ident
            );
        }

        // TODO fn: This is scheduled for removal
        #[test]
        fn new_for_version() {
            let ident =
                Ident::new("neal-morse-band", "long-day", Some("9.0.9"), None::<&str>).unwrap();

            assert_eq!(
                Ident::Version(VersionIdent::from_str("neal-morse-band/long-day/9.0.9").unwrap()),
                ident
            );
        }

        // TODO fn: This is scheduled for removal
        #[test]
        fn new_for_name() {
            let ident =
                Ident::new("neal-morse-band", "long-day", None::<&str>, None::<&str>).unwrap();

            assert_eq!(
                Ident::Name(NameIdent::from_str("neal-morse-band/long-day").unwrap()),
                ident
            );
        }

        // TODO fn: This is scheduled for removal
        #[test]
        fn new_for_invalid() {
            let invalid = "neal-morse-band/long-day//20180810161823";

            match Ident::new(
                "neal-morse-band",
                "long-day",
                None::<&str>,
                Some("20180810161823"),
            ) {
                Err(Error::InvalidPackageIdent(ref val)) => assert_eq!(val, invalid),
                Err(e) => panic!("Ident::new failed with wrong error type: {}", e),
                Ok(_) => panic!("Ident::new should fail to parse: {}", invalid),
            }
        }
    }
}
