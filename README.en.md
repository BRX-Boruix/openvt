# openvt

A BORUIX terminal utility: request a new terminal instance at run time.

[简体中文](README.md)

## Usage

Run from the shell; no arguments:

```
[openvt] requesting a new console instance
[openvt] request queued for instance 4; wait for init patrol log (instance N created)
```

The program finds a free instance number itself: it tries upward from 1, skipping numbers already
in use. It submits the request and returns — terminal creation is asynchronous, done by the system
init process; the creation log line printed by init is the evidence that the terminal is usable.

## Behaviour

- The instance number is capped at 64, matching the system limit on terminals
- Boot-time pre-created instances and already-submitted requests are both skipped
- When every number is taken it honestly reports "no free console slot" and exits non-zero
- Failures print the error category (not found, permission denied, already exists), never a bare failure

## Known limitations

- The instance number cannot be chosen; it is assigned by the program
- A submitted request cannot be cancelled

## Building

```bash
cargo build --release
```

## Repository layout

```
openvt/
├── Cargo.toml    # package manifest
├── build.rs      # injects the linker script
├── linker.ld     # user-space segment layout
└── src/
    └── main.rs   # instance-number probing and request submission
```

## Related projects

- [`init`](https://github.com/BRX-Boruix/init) — consumes the request and creates the terminal
- [`consoled`](https://github.com/BRX-Boruix/consoled) — supplies the byte stream for the new terminal
- [`login`](https://github.com/BRX-Boruix/login) — provides login on the new terminal

## License

MIT License, copyright Yang Borui. See [LICENSE](LICENSE).
