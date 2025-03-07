use std::fs::{read_to_string, File};
use std::io::Result;

use stdio_utils::{null, Duplicate as _, Guard};

fn main() -> Result<()> {
    println!("Now you see me");

    // redirect stdout to /dev/null
    let guard = Guard::stdout()?;
    null()?.duplicate_to_stdout()?;
    println!("Now you don't");

    // restore stdout to the console
    drop(guard);
    println!("Now you see me again");

    // redirect stdout to ./output.txt
    let guard = Guard::stdout()?;
    File::create("./output.txt")?.duplicate_to_stdout()?;
    println!("Now you see me if you search");

    // restore stdout to the console
    drop(guard);

    let msg = read_to_string("./output.txt")?;
    println!("{msg:?}");

    Ok(())
}
