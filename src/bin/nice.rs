//! A sketchy implementation of the `nice` utility built on `scrummage`.
use scrummage::{Priority, Process};
use std::process::Command;

macro_rules! fail {
    ($n:literal : $fmt:literal $(, $t:expr)*) => {|| {
        // TODO: Fill help message
        eprintln!(concat!("help blah blah\n", $fmt) $(, $t)*);
        std::process::exit($n);
    }}
}

fn main() {
    let mut args = std::env::args_os();
    let first = args.nth(1).unwrap_or_else(fail!(1: "expected a `utility`"));
    let mut child = if let Some("-n") = first.to_str() {
        let arg = args
            .next()
            .unwrap_or_else(fail!(1: "expected an `increment`"));
        let priority = arg
            .to_str()
            .and_then(|s| s.parse().ok())
            .map(|n: i64| {
                if n >= 0 {
                    Priority::normal()
                        .lower()
                        .take(n as usize)
                        .last()
                        .unwrap_or(Priority::normal())
                } else {
                    Priority::normal()
                        .higher()
                        .take(-n as usize)
                        .last()
                        .unwrap_or(Priority::normal())
                }
            })
            .unwrap_or_else(fail!(1: "{:?} is not an `increment`", arg));

        let cmd = args.next().unwrap_or_else(fail!(1: "expected a `utility`"));
        let mut child = Command::new(&cmd)
            .args(args)
            .spawn()
            .ok()
            .unwrap_or_else(fail!(127: "something went wrong while running {:?}", cmd));
        if let Err(e) = Process::from(&mut child).set_priority(priority) {
            eprintln!("Failed to set priority: {}", e);
        }
        child
    } else {
        Command::new(&first)
            .args(args)
            .spawn()
            .ok()
            .unwrap_or_else(fail!(127: "something went wrong while running {:?}", first))
    };
    let n = child
        .wait()
        // TODO: Propagate signals
        .map(|status| status.code().unwrap())
        .unwrap();
    std::process::exit(n);
}
