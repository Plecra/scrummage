# scrummage

Control the scheduling of your processes to make your programs more reponsive!

```rust
scrummage::Process::current()
    .set_priority(scrummage::Priority::low())
    .expect("couldn't set own thread priority");
```

## Roadmap

- [x] Linux support
    - [ ] ...then Unix,
    - [ ] and MacOS?
- [ ] Windows support
- [ ] Thread prioritisation
    - This is currently part of [thread-priority]
    - and it's harder to get right; I'd like to protect users from Priority
      Inversion etc.

[thread-priority]: https://crates.io/crates/thread-priority
