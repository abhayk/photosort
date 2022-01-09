# photosort
Sort photos, to a target directory, based on their exif timestamp. If the file does not contain exif data or if there an error reading the exif the file modified time is used instead.

The exif tag `OriginalDateTime` is used to determine the timestamp. Supported file types are `jpeg`, `png` and `tiff`.

For example a file with the exif time as Jan 9th 2022 will end up in the target directory as - 
```
<TARGET_DIR>
└── 2022/
   └── January/
       └── 9/
           └── image.jpg
```

If a file is already present in the destination then it is not copied. 

# Usage
The latest version can be downloaded from the [releases](https://github.com/abhayk/photosort/releases) page.

```
photosort 0.1.0
photosort is a tool used to sort photos into a target directory
based on their exif timestamp.

USAGE:
    photosort.exe --source-dir <SOURCE_DIR> --target-dir <TARGET_DIR>

OPTIONS:
    -h, --help                       Print help information
    -s, --source-dir <SOURCE_DIR>
    -t, --target-dir <TARGET_DIR>
    -V, --version                    Print version information
```

# Credits
- The [exif-rs](https://github.com/kamadak/exif-rs) library for parsing exif.
- The [exif-samples](https://github.com/ianare/exif-samples) repository for sample images to test the exif parsing.