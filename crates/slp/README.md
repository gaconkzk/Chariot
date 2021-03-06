Chariot SLP Crate
-----------------

This crate handles the SLP file format used by Age of Empires (1997).
Currently, it can read SLP files.

While the ability to write an SLP file is a nice to have, it's not strictly
necessary for the rest of the Chariot project, and thus, is not implemented
at this time.

The code herein falls under the same license as the rest of the Chariot project.

### Building

You'll need the Rust compiler and Cargo build system. Once you have those,
you can compile with:

```
$ cargo build
```

### Example

```rust,norun
match slp::SlpFile::read_from_file("/path/to/file.slp", 1u8) {
    Ok(slp_file) => {
        println!("Shape count: {}", slp_file.header.shape_count);
        for shape in &slp_file.shapes {
            println!("{:?}", shape.header);
        }
    },
    Err(err) => {
        println!("Failed to read the SLP file: {}", err);
    }
}
```

### SLP format in ASCII form
```
+-----------------------------+
|          SlpHeader          |
+-----------------------------+
|SlpShapeHeader|SlpShapeHeader|
+-----------------------------+
|                             |
| Array of u16 padding pairs  | <-+ Each SlpShapeHeader has a "shape_outline_offset"
|                             |     that points to a pair in this array
+-----------------------------+
|                             |
| Arrays of u32 offsets to    | <-+ Each SlpShapeHeader has a "shape_data_offsets"
|  first command in each row  |     that points to an array
|                             |
+-----------------------------+
|                             |
| Drawing commands used to    |
|  produce indexed image data |
|                             |
+-----------------------------+
```
