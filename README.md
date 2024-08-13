# Building

Just run `cargo`.

```ps
cargo build --release
```

# Usage
```ps1
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

### If you want to patch `system.dat` (the main archive with BGM, graphics etc.)

```ps1
.\laby2repack.exe --unpack [path to system.dat] [path to asset folder]

# replace your game files in the asset folder

.\laby2repack.exe --pack [path to asset folder] [path to system.dat]
```
