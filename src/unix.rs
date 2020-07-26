use crate::{Error, NotFound};
use libc::{getpid, getpriority, setpriority, PRIO_PROCESS};

#[derive(Debug)]
pub(crate) struct Process<'a> {
    // FIXME: getpid returns an i32, but s/getpriority take a u32. What am I
    // meant to store? I *think* the casts should retain the meaning anyway,
    // but that should be checked.
    pid: u32,
    marker: core::marker::PhantomData<&'a ()>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Priority {
    niceness: libc::c_int,
}

impl Priority {
    pub fn higher(&self) -> impl Iterator<Item = Self> {
        let mut niceness = self.niceness;
        core::iter::from_fn(move || {
            if niceness > -20 {
                niceness -= 1;
                Some(Self { niceness })
            } else {
                None
            }
        })
    }
    pub fn normal() -> Self {
        Self { niceness: 0 }
    }
    pub fn lower(&self) -> impl Iterator<Item = Self> {
        let mut niceness = self.niceness;
        core::iter::from_fn(move || {
            if niceness < 19 {
                niceness += 1;
                Some(Self { niceness })
            } else {
                None
            }
        })
    }
}

fn unexpected_err(errno: i32) -> ! {
    unreachable!("unexpected error: {}", {
        #[cfg(feature = "std")]
        {
            std::io::Error::from_raw_os_error(errno)
        }
        #[cfg(not(feature = "std"))]
        {
            errno
        }
    })
}
fn errno() -> i32 {
    // Safety: errno is thread-local, and __errno_location will
    // always return a valid reference
    unsafe { *libc::__errno_location() }
}
impl Process<'_> {
    pub fn current() -> Process<'static> {
        Process {
            // Safety: `getpid` is always safe to call
            pid: unsafe { getpid() } as u32,
            marker: core::marker::PhantomData,
        }
    }
    pub fn set_priority(&mut self, priority: Priority) -> Result<(), Error> {
        // Safety: `setpriority` checks its arguments
        if unsafe { setpriority(PRIO_PROCESS, self.pid, priority.niceness) } == 0 {
            Ok(())
        } else {
            match errno() {
                libc::ESRCH => Err(Error::NotFound(NotFound)),
                libc::EACCES | libc::EPERM => Err(Error::NotAllowed),
                errno => unexpected_err(errno),
            }
        }
    }
    pub fn priority(&self) -> Result<Priority, NotFound> {
        // `getpriority` doesn't return an error code, so we need
        // to reset `errno` in advance
        unsafe {
            // Safety: errno is thread-local, and __errno_location will
            // always return a valid reference
            *libc::__errno_location() = 0;
        }
        // Safety: `getpriority` checks its arguments
        let niceness = unsafe { getpriority(PRIO_PROCESS, self.pid) };
        match errno() {
            0 => Ok(Priority { niceness }),
            libc::ESRCH => Err(NotFound),
            errno => unexpected_err(errno),
        }
    }
}

#[cfg(feature = "std")]
impl<'a> From<&'a mut std::process::Child> for Process<'a> {
    fn from(child: &mut std::process::Child) -> Self {
        Self {
            pid: child.id() as u32,
            marker: core::marker::PhantomData,
        }
    }
}
