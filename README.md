# Dragon Dance Trace Recorder

Record code coverage traces in rust to the
[dragondance](https://github.com/0ffffffffh/dragondance) format.

[Docs](https://docs.rs/dragondance)

## Example

```rust
use dragondance::{Module, Trace};

// Create a Trace with module info
let modules = [Module::new("abcd", 0x1000, 0x2000),
               Module::new("libc.so", 0x555000, 0x556000)];
let mut trace = Trace::new(&modules);

// Add coverage events from your emulator, debugger, etc.
trace.add(0x1204, 3);
trace.add(0x1207, 12);

// Write the coverage to a dragondance coverage file
trace.save("trace.dd").unwrap();
```
