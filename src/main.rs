use std::{fs, io, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use id3::{
    frame::{Chapter, Picture, PictureType},
    Tag, TagLike, Version,
};
use tempfile::NamedTempFile;

const MERGELIST_PATH: &str = "mergelist.txt";

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to cover art
    #[clap(short, long)]
    cover: Option<PathBuf>,
    /// Output file path
    output: PathBuf,
    /// Input file paths
    files: Vec<String>,
}

fn get_chapters(args: &Args) -> anyhow::Result<Vec<Chapter>> {
    let mut chapters = vec![];

    let mut current_time: u32 = 0;
    let mut current_offset: u32 = 0;

    for (i, path) in args.files.iter().enumerate() {
        let element_id = format!("chapter_{i}");

        let duration_secs: f64 = duct::cmd!(
            "ffprobe",
            "-i",
            path,
            "-show_entries",
            "format=duration",
            "-v",
            "quiet",
            "-of",
            "csv=p=0"
        )
        .read()
        .with_context(|| format!("unable to get duration of input file '{path}'"))?
        .parse()
        .with_context(|| format!("unable to parse duration of input file '{path}'"))?;

        let duration_ms = (duration_secs * 1000.0).round() as u32;

        let file_size = fs::metadata(path)
            .with_context(|| format!("unable to get size of input file '{path}'"))?
            .len() as u32;

        let mut chapter = Chapter {
            element_id,
            start_time: current_time,
            end_time: current_time + duration_ms,
            start_offset: current_offset,
            end_offset: current_offset + file_size,
            frames: vec![],
        };

        chapter.set_title(
            PathBuf::from(path)
                .file_stem()
                .with_context(|| {
                    format!("unable to determine chapter title for input file '{path}'")
                })?
                .to_string_lossy(),
        );

        current_time += duration_ms;
        current_offset += file_size;

        chapters.push(chapter);
    }

    Ok(dbg!(chapters))
}

fn create_mergelist(args: &Args) -> io::Result<()> {
    let lines: Vec<_> = args
        .files
        .iter()
        .map(|path| path.replace('\'', "'\\''"))
        .map(|path| {
            if PathBuf::from(&path).is_relative() {
                format!("file './{path}'")
            } else {
                format!("file '{path}'")
            }
        })
        .collect();

    println!("{}", lines.join("\n"));

    fs::write(MERGELIST_PATH, lines.join("\n"))
}

fn merge_files() -> io::Result<NamedTempFile> {
    let merged_file = tempfile::Builder::new()
        .prefix("merge-output")
        .suffix(".mp3")
        .tempfile()?;

    let _output = duct::cmd!(
        "ffmpeg",
        "-f",
        "concat",
        "-safe",
        "0",
        "-i",
        MERGELIST_PATH,
        "-c",
        "copy",
        "-y",
        merged_file.path()
    )
    // .stderr_to_stdout()
    .run()?;

    fs::remove_file(MERGELIST_PATH)?;

    Ok(merged_file)
}

fn add_chapters(metadata: &mut Tag, chapters: Vec<Chapter>) -> anyhow::Result<()> {
    for chapter in chapters {
        metadata.add_frame(chapter);
    }

    Ok(())
}

fn add_cover(metadata: &mut Tag, path: &PathBuf) -> anyhow::Result<()> {
    let mime_type = mime_guess::from_path(path).first().with_context(|| {
        format!(
            "unable to determine a mime type for cover file '{}'",
            path.to_string_lossy()
        )
    })?;

    let image_data = fs::read(path)
        .with_context(|| format!("unable to read cover file '{}'", path.to_string_lossy()))?;

    metadata.add_frame(Picture {
        mime_type: mime_type.to_string(),
        picture_type: PictureType::CoverFront,
        description: String::new(),
        data: image_data,
    });

    Ok(())
}

// Basic steps:
// - Determine length of each file and create chapter info
// - Merge chosen files into a single MP3
// - Write chapter info + optional cover image to merged MP3
fn main() -> anyhow::Result<()> {
    let args: Args = dbg!(Args::parse());

    let chapters = get_chapters(&args)?;
    create_mergelist(&args)?;
    let merged_file = merge_files()?;

    let mut metadata = Tag::read_from_path(merged_file.path())?;
    add_chapters(&mut metadata, chapters)?;
    if let Some(ref path) = args.cover {
        add_cover(&mut metadata, path)?;
    }

    metadata.write_to_path(merged_file.path(), Version::Id3v24)?;
    fs::copy(merged_file.path(), args.output)?;

    Ok(())
}
