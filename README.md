## Building

Use nightly `rustc` to build:

`cargo +nightly build --release`

## Usage
```
Usage: laby2repack.exe (-u | -p) <input> <output>

Available positional items:
    <input>       Input path.
                  When packing, point this to the directory with unpacked files.
                  When unpacking, point this to the game archive file.
    <output>      Output path.
                  When packing, enter the output file path.
                  When unpacking, enter the output directory path.

Available options:
    -u, --unpack  Unpack the game files to a directory
    -p, --pack    Pack existing game files to an archive
    -h, --help    Prints help information
```
