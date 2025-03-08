use std::fs::{read_to_string, File};
use std::io::Result;

use stdio_utils::{null, StdioOverride as _};

fn main() -> Result<()> {
    println!("Now you see me");

    // redirect stdout to /dev/null
    {
        let _guard = null()?.override_stdout()?;
        println!("Now you don't");
    }

    // stdout to the console is restored
    println!("Now you see me again");

    // redirect stdout to ./output.txt
    {
        let _guard = File::create("./output.txt")?.override_stdout()?;
        println!("Now you see me if you search");
    }

    // stdout to the console is restored
    let msg = read_to_string("./output.txt")?;
    println!("{msg:?}");

    Ok(())
}
