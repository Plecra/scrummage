#![cfg_attr(not(feature = "std"), no_std)]
//! A thin abstraction over OS process scheduling APIs.
//!
//! By signalling the priority of our processes to the operating system, we
//! gain more control over our program's resource usage, and which tasks get
//! completed first.
//!
//! For example, we can configure our UI to preempt background tasks by giving
//! it a higher priority:
//!
//! ```rust
//! # use scrummage::{Process, Priority};
//! # // TODO: Find a better cross-platform shell program
//! # let mut busy_child_process = std::process::Command::new("cargo").arg("version").spawn().unwrap();
//! let me = Process::current().priority().unwrap();
//! let boring_work = me.lower().next().expect("no lower priority available");
//! // It's fine if the `busy_child_process` has already finished
//! let _ = Process::from(&mut busy_child_process)
//!     .set_priority(boring_work);
//! ```
//!
//! This will tell the OS to make sure `me` is always given all the resources
//! it needs, making it snappier. 

macro_rules! doctest {
    ($x:expr) => {
        #[doc = $x]
        extern {}
    };
}
doctest!(include_str!("../README.md"));

#[cfg_attr(windows, path = "./windows.rs")]
#[cfg_attr(unix, path = "./unix.rs")]
mod imp;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A prioritisation level
///
/// The priority of a [`Process`] controls how much CPU time it gets
/// compared to other processes. Most programs don't need to be handled
/// especially, and should be given a [normal](Priority::normal) priority
/// to allow the OS to handle scheduling
pub struct Priority(imp::Priority);

impl Priority {
    // TODO: consider declaring these as `const fn`
    /// The priority level given to normal processes; The default priority
    /// level.
    ///
    /// ```rust
    /// # use scrummage::{Process, Priority};
    /// assert_eq!(Process::current().priority().unwrap(), Priority::normal(),
    ///            "I'm normal! Normal I tell you!");
    /// ```
    pub fn normal() -> Self {
        Self(imp::Priority::normal())
    }
    /// Raise the priority level.
    ///
    /// Be particularly careful with giving processes higher priority levels:
    /// Any process with a lower level will be halted until it pauses.
    /// Therefore, make sure any work it does is breif, and it uses OS APIs for
    /// delays ([`std::thread::sleep`] instead of `loop {}`)
    pub fn higher(&self) -> impl Iterator<Item = Self> {
        self.0.higher().map(Self)
    }
    /// Lower the priority level.
    ///
    /// Processes with lower priority levels will pause if other processes need
    /// to do work. They can be used for screen-savers e.t.c.
    pub fn lower(&self) -> impl Iterator<Item = Self> {
        self.0.lower().map(Self)
    }
}

#[derive(Debug)]
/// A process running on this machine.
///
/// Because the OS owns the process this "refers" to, we can't know it's valid:
/// someone could've killed it. Therefore, the methods return [`NotFound`] if
/// they are ever called on a dead process.
pub struct Process<'a>(imp::Process<'a>);

impl Process<'_> {
    /// Get the currently running process
    ///
    /// Note that this is will last for `'static`, since the OS process it
    /// refers to contains this very struct, and if it died, then this struct
    /// must have died with it.
    pub fn current() -> Process<'static> {
        Process(imp::Process::current())
    }
    /// Update the priority of this process

    pub fn set_priority(&mut self, priority: Priority) -> Result<(), Unchanged> {
        self.0.set_priority(priority.0)
    }
    /// Fetch the priority of this process
    pub fn priority(&self) -> Result<Priority, NotFound> {
        self.0.priority().map(Priority)
    }
}

// TODO: This API sorta sucks. Would Process::of_child(&Child) be better?
// The name's less than obvious
#[cfg(feature = "std")]
impl<'a> From<&'a mut std::process::Child> for Process<'a> {
    fn from(child: &'a mut std::process::Child) -> Self {
        Self(child.into())
    }
}

/// The process couldn't be found.
///
/// See [`Process`] for details.
#[derive(Debug)]
pub struct NotFound;

/// The reason the priority of a process couldn't be set.
#[derive(Debug)]
pub enum Unchanged {
    // This could be much cleaner with [enum variant types], which would
    // let `Process::priority` return `Result<Priority, Error::NotFound>`
    //
    // [enum variant types]: https://github.com/rust-lang/rfcs/pull/2593
    NotFound(NotFound),
    /// The [`Process`] handle didn't have the suitable permissions to
    /// set priority.
    /// 
    /// Each platform has a set of rules around who can set whose priority,
    /// and you should check the documentation for your platform to make sure
    /// you are setting up the right permissions. If the details of this error
    /// would be useful for you, do file an issue about your use case! 😁 
    PermissionDenied,
}

impl From<NotFound> for Unchanged {
    fn from(n: NotFound) -> Self {
        Self::NotFound(n)
    }
}

impl core::fmt::Display for NotFound {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("couldn't set priority of missing process")
    }
}

impl core::fmt::Display for Unchanged {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::NotFound(n) => core::fmt::Display::fmt(n, f),
            Self::PermissionDenied => f.write_str("missing permissions to set priority"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NotFound {}

#[cfg(feature = "std")]
impl std::error::Error for Unchanged {}
