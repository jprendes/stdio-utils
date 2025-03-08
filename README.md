# stdio-utils

A set of cross-platform utilities for handling the standard input output in Rust.

```rust
use std::fs::{read_to_string, File};
use std::io::Result;

use stdio_utils::{null, StdioOverride as _};

fn main() -> Result<()> {
    println!("Now you see me");

    // redirect stdout to /dev/null
    let guard = null()?.override_stdout()?;
    println!("Now you don't");

    // restore stdout to the console
    drop(guard);
    println!("Now you see me again");

    // redirect stdout to ./output.txt
    let guard = File::create("./output.txt")?.override_stdout()?;
    println!("Now you see me if you search");

    // restore stdout to the console
    drop(guard);

    let msg = read_to_string("./output.txt")?;
    println!("{msg:?}");

    Ok(())
}
```

This should print
```
Now you see me
Now you see me again
"Now you see me if you search\n"
```