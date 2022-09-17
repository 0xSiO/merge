# merge

Merge audio files into a single MP3 file, with chapters and optional cover art.

## Usage

Make sure `ffmpeg` and `ffprobe` are installed and available in your PATH.

```
USAGE:
    merge [OPTIONS] <OUTPUT> [FILES]...

ARGS:
    <OUTPUT>      Output file path
    <FILES>...    Input file paths

OPTIONS:
    -c, --cover <COVER>    Path to cover art
    -h, --help             Print help information
    -t, --title <TITLE>    Set title of merged MP3 file
    -V, --version          Print version information
```

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/0xSiO/merge.

## License

This crate is available as open source under the terms of the
[MIT License](https://opensource.org/licenses/MIT).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
`merge` by you shall be licensed as MIT, without any additional terms or conditions.
