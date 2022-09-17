# merge

Merge audio files into a single MP3 file, with chapters and optional metadata.

## Usage

Make sure [`ffmpeg`](https://ffmpeg.org/ffmpeg.html) and
[`ffprobe`](https://ffmpeg.org/ffprobe.html) are installed and available in your PATH.
I've tested this with `ffmpeg`/`ffprobe` v5.0.1, but other versions might work too.

```
USAGE:
    merge [OPTIONS] <OUTPUT> [FILES]...

ARGS:
    <OUTPUT>      Output file path
    <FILES>...    Input file paths

OPTIONS:
        --album <ALBUM>                    Album name
        --album-artist <ALBUM_ARTIST>      Album artist
        --artists <ARTISTS>                Semicolon-separated list of artists
        --comments <COMMENTS>              Comments to include
        --cover <COVER>                    Path to cover art image
        --date-released <DATE_RELEASED>    Date released
        --genres <GENRES>                  Semicolon-separated list of genres
    -h, --help                             Print help information
        --subtitle <SUBTITLE>              Set subtitle of merged MP3 file
        --title <TITLE>                    Set title of merged MP3 file
    -V, --version                          Print version information
```

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/0xSiO/merge.

## License

This crate is available as open source under the terms of the
[MIT License](https://opensource.org/licenses/MIT).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
`merge` by you shall be licensed as MIT, without any additional terms or conditions.
