use crate::{Unchanged, NotFound};
use winapi::um::winnt::HANDLE;
use winapi::um::processthreadsapi::{GetCurrentProcess, GetPriorityClass, SetPriorityClass};
use winapi::um::winbase;
use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;

#[derive(Debug)]
pub(crate) struct Process<'a> {
    // FIXME: getpid returns an i32, but s/getpriority take a u32. What am I
    // meant to store? I *think* the casts should retain the meaning anyway,
    // but that should be checked.
    handle: HANDLE,
    marker: core::marker::PhantomData<&'a ()>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Priority {
    priority_class: DWORD,
}

impl Priority {
    pub fn higher(&self) -> impl Iterator<Item = Self> {
        let mut priority_class = self.priority_class;
        core::iter::from_fn(move || {
            match priority_class {
                winbase::IDLE_PRIORITY_CLASS => Some(winbase::BELOW_NORMAL_PRIORITY_CLASS),
                winbase::BELOW_NORMAL_PRIORITY_CLASS => Some(winbase::NORMAL_PRIORITY_CLASS),
                winbase::NORMAL_PRIORITY_CLASS => Some(winbase::ABOVE_NORMAL_PRIORITY_CLASS),
                winbase::ABOVE_NORMAL_PRIORITY_CLASS => Some(winbase::HIGH_PRIORITY_CLASS),
                winbase::HIGH_PRIORITY_CLASS => Some(winbase::REALTIME_PRIORITY_CLASS),
                winbase::REALTIME_PRIORITY_CLASS => None,
                _ => unreachable!("invalid priority class found")
            }.map(|pc| {
                priority_class = pc;
                Self { priority_class }
            })
        })
    }
    pub fn normal() -> Self {
        Self { priority_class: winbase::NORMAL_PRIORITY_CLASS }
    }
    pub fn lower(&self) -> impl Iterator<Item = Self> {
        let mut priority_class = self.priority_class;
        core::iter::from_fn(move || {
            match priority_class {
                winbase::REALTIME_PRIORITY_CLASS => Some(winbase::HIGH_PRIORITY_CLASS),
                winbase::HIGH_PRIORITY_CLASS => Some(winbase::ABOVE_NORMAL_PRIORITY_CLASS),
                winbase::ABOVE_NORMAL_PRIORITY_CLASS => Some(winbase::NORMAL_PRIORITY_CLASS),
                winbase::NORMAL_PRIORITY_CLASS => Some(winbase::BELOW_NORMAL_PRIORITY_CLASS),
                winbase::BELOW_NORMAL_PRIORITY_CLASS => Some(winbase::IDLE_PRIORITY_CLASS),
                winbase::IDLE_PRIORITY_CLASS => None,
                _ => unreachable!("invalid priority class found")
            }.map(|pc| {
                priority_class = pc;
                Self { priority_class }
            })
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

impl Process<'_> {
    pub fn current() -> Process<'static> {
        Process {
            // Safety: `getpid` is always safe to call
            handle: unsafe { GetCurrentProcess() },
            marker: core::marker::PhantomData,
        }
    }
    pub fn set_priority(&mut self, priority: Priority) -> Result<(), Unchanged> {
        // Safety: `SetPriorityClass` checks its arguments
        if unsafe { SetPriorityClass(self.handle, priority.priority_class) } == 0 {
            match unsafe { GetLastError() } {
                errno => unexpected_err(errno),
            }
        } else {
            Ok(())
        }
    }
    pub fn priority(&self) -> Result<Priority, NotFound> {
        // Safety: `GetPriorityClass` checks its arguments
        match unsafe { GetPriorityClass(self.handle) } {
            0 => Err(NotFound),
            priority_class => Ok(Priority { priority_class })
        }
    }
}

#[cfg(feature = "std")]
impl<'a> From<&'a mut std::process::Child> for Process<'a> {
    fn from(child: &mut std::process::Child) -> Self {
        Self {
            handle: std::os::windows::io::AsRawHandle::as_raw_handle(child),
            marker: core::marker::PhantomData,
        }
    }
}
