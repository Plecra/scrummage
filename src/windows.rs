use crate::{Unchanged, NotFound};
use winapi::um::processthreadsapi::{
    GetPriorityClass,
    SetPriorityClass,
    GetCurrentProcess,
};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::minwindef::DWORD;

#[derive(Debug)]
pub(crate) struct Process<'a> {
    handle: HANDLE,
    marker: core::marker::PhantomData<&'a ()>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Priority {
    priority: DWORD,
}
impl Priority {
    fn to_relative(&self) -> u8 {
        match self.priority {
            winbase::IDLE_PRIORITY_CLASS => 0,
            winbase::BELOW_NORMAL_PRIORITY_CLASS => 1,
            winbase::NORMAL_PRIORITY_CLASS | winbase::PROCESS_MODE_BACKGROUND_BEGIN | winbase::PROCESS_MODE_BACKGROUND_END => 2,
            winbase::ABOVE_NORMAL_PRIORITY_CLASS => 3,
            winbase::HIGH_PRIORITY_CLASS => 4,
            winbase::REALTIME_PRIORITY_CLASS => 5,
            n => unreachable!("undefined priority {}", n),
        }
    }
}
impl core::cmp::PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl core::cmp::Ord for Priority {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_relative().cmp(&other.to_relative())
    }
}

impl Priority {
    pub fn higher(&self) -> impl Iterator<Item = Self> {
        let mut priority = self.priority;
        core::iter::from_fn(move || {
            match priority {
                winbase::IDLE_PRIORITY_CLASS => Some(winbase::BELOW_NORMAL_PRIORITY_CLASS),
                winbase::BELOW_NORMAL_PRIORITY_CLASS => Some(winbase::NORMAL_PRIORITY_CLASS),
                winbase::NORMAL_PRIORITY_CLASS => Some(winbase::ABOVE_NORMAL_PRIORITY_CLASS),
                winbase::ABOVE_NORMAL_PRIORITY_CLASS => Some(winbase::HIGH_PRIORITY_CLASS),
                winbase::HIGH_PRIORITY_CLASS => Some(winbase::REALTIME_PRIORITY_CLASS),
                _ => None
            }.map(|n| {
                priority = n;
                Priority { priority: n }
            })
        })
    }
    pub fn normal() -> Self {
        Self { priority: winbase::NORMAL_PRIORITY_CLASS }
    }
    pub fn lower(&self) -> impl Iterator<Item = Self> {
        let mut priority = self.priority;
        core::iter::from_fn(move || {
            match priority {
                winbase::BELOW_NORMAL_PRIORITY_CLASS => Some(winbase::IDLE_PRIORITY_CLASS),
                winbase::NORMAL_PRIORITY_CLASS => Some(winbase::BELOW_NORMAL_PRIORITY_CLASS),
                winbase::ABOVE_NORMAL_PRIORITY_CLASS => Some(winbase::NORMAL_PRIORITY_CLASS),
                winbase::HIGH_PRIORITY_CLASS => Some(winbase::ABOVE_NORMAL_PRIORITY_CLASS),
                winbase::REALTIME_PRIORITY_CLASS => Some(winbase::HIGH_PRIORITY_CLASS),
                _ => None,
            }.map(|n| {
                priority = n;
                Priority { priority: n }
            })
        })
    }
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
        // Safety: `self.handle` is a valid handle
        if dbg!(unsafe { SetPriorityClass(self.handle, dbg!(priority.priority)) }) == 0 {
            // Safety: GetLastError is thread-local
            match unsafe { GetLastError() } {
                errcode => todo!("handle relevant error: {}", errcode),
            }
        } else {
            dbg!(self.priority(), priority);
            Ok(())
        }
    }
    pub fn priority(&self) -> Result<Priority, NotFound> {
        // Safety: `self.handle` is a valid handle
        let priority = unsafe { GetPriorityClass(self.handle) };
        if priority == 0 {
            Err(NotFound)
        } else {
            Ok(Priority { priority })
        }
    }
}

#[cfg(feature = "std")]
impl<'a> From<&'a mut std::process::Child> for Process<'a> {
    fn from(child: &mut std::process::Child) -> Self {
        use std::os::windows::io::AsRawHandle;
        Self {
            handle: child.as_raw_handle(),
            marker: core::marker::PhantomData,
        }
    }
}
