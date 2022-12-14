use std::{fs, io, path::PathBuf, time::Duration};

use anyhow::Context;
use chrono::{Datelike, NaiveDate};
use clap::Parser;
use id3::{
    frame::{Chapter, Comment, Picture, PictureType},
    Tag, TagLike, Timestamp, Version,
};
use indicatif::{ProgressBar, ProgressStyle};
use tempfile::NamedTempFile;

// We can't use a temporary path for the mergelist, unfortunately. ffmpeg considers relative paths
// in the mergelist to be relative to the location of the mergelist, rather than the current
// working directory.
const MERGELIST_PATH: &str = "mergelist.txt";

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Set title of merged MP3 file
    #[clap(long)]
    title: Option<String>,
    /// Set subtitle of merged MP3 file
    #[clap(long)]
    subtitle: Option<String>,
    /// Semicolon-separated list of artists
    #[clap(long)]
    artists: Option<String>,
    /// Path to cover art image
    #[clap(long)]
    cover: Option<String>,
    /// Album name
    #[clap(long)]
    album: Option<String>,
    /// Album artist
    #[clap(long)]
    album_artist: Option<String>,
    /// Date released
    #[clap(long)]
    date_released: Option<String>,
    /// Semicolon-separated list of genres
    #[clap(long)]
    genres: Option<String>,
    /// Comments to include
    #[clap(long)]
    comments: Option<String>,
    /// Output file path
    output: PathBuf,
    /// Input file paths
    files: Vec<String>,
}

fn get_chapters(args: &Args) -> anyhow::Result<Vec<Chapter>> {
    let mut chapters = Vec::with_capacity(args.files.len());
    let mut current_time: u32 = 0;
    let mut current_offset: u32 = 0;

    let progress_bar = ProgressBar::new(args.files.len() as u64)
        .with_style(ProgressStyle::default_bar().template("[{pos}/{len}] {spinner} {msg}")?);
    progress_bar.enable_steady_tick(Duration::from_millis(100));

    for (i, path) in args.files.iter().enumerate() {
        progress_bar.inc(1);
        progress_bar.set_message(format!("📖 generating chapter info for '{path}'..."));

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
        .with_context(|| format!("failed to get duration of input file '{path}'"))?
        .parse()
        .with_context(|| format!("failed to parse duration of input file '{path}'"))?;

        let duration_ms = (duration_secs * 1000.0).round() as u32;

        let file_size = fs::metadata(path)
            .with_context(|| format!("failed to get info for input file '{path}'"))?
            .len() as u32;

        let mut chapter = Chapter {
            element_id: format!("chapter_{i}"),
            start_time: current_time,
            end_time: current_time + duration_ms,
            start_offset: current_offset,
            end_offset: current_offset + file_size,
            frames: vec![],
        };

        chapter.set_title(
            PathBuf::from(path)
                .file_stem()
                .with_context(|| format!("failed to get stem for input file '{path}'"))?
                .to_string_lossy(),
        );

        current_time += duration_ms;
        current_offset += file_size;

        chapters.push(chapter);
    }

    progress_bar.set_message("📕 chapter info generated!");
    progress_bar.finish();

    Ok(chapters)
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

    fs::write(MERGELIST_PATH, lines.join("\n"))
}

fn merge_files() -> io::Result<NamedTempFile> {
    let merged_file = tempfile::Builder::new()
        .prefix("merge-output")
        .suffix(".mp3")
        .tempfile()?;

    let progress_bar = ProgressBar::new_spinner().with_message("🔨 merging input files...");
    progress_bar.enable_steady_tick(Duration::from_millis(100));

    let _output = duct::cmd!(
        "ffmpeg",
        "-hide_banner",
        "-loglevel",
        "error",
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
    .run()?;

    progress_bar.finish_with_message("💽 merged!");

    fs::remove_file(MERGELIST_PATH)?;

    Ok(merged_file)
}

fn populate_metadata(
    args: &Args,
    metadata: &mut Tag,
    chapters: Vec<Chapter>,
) -> anyhow::Result<()> {
    if let Some(title) = &args.title {
        metadata.set_title(title);
    }

    if let Some(subtitle) = &args.subtitle {
        metadata.set_text("TIT3", subtitle);
    }

    if let Some(artists) = &args.artists {
        metadata.set_text_values("TPE1", artists.split(';'))
    }

    if let Some(path) = &args.cover {
        let mime_type = mime_guess::from_path(path).first().with_context(|| {
            format!("failed to determine a mime type for cover file '{}'", path)
        })?;

        let image_data =
            fs::read(path).with_context(|| format!("failed to read cover file '{}'", path))?;

        metadata.add_frame(Picture {
            mime_type: mime_type.to_string(),
            picture_type: PictureType::CoverFront,
            description: String::new(),
            data: image_data,
        });
    }

    if let Some(album) = &args.album {
        metadata.set_album(album);
    }

    if let Some(album_artist) = &args.album_artist {
        metadata.set_album_artist(album_artist);
    }

    if let Some(date_released) = &args.date_released {
        let parsed_date = NaiveDate::parse_from_str(date_released, "%Y-%m-%d")
            .with_context(|| format!("failed to parse release date timestamp '{date_released}'"))?;

        metadata.set_date_released(Timestamp {
            year: parsed_date.year(),
            month: Some(parsed_date.month() as u8),
            day: Some(parsed_date.day() as u8),
            hour: None,
            minute: None,
            second: None,
        });
    }

    if let Some(genres) = &args.genres {
        metadata.set_text_values("TCON", genres.split(';'));
    }

    if let Some(comments) = &args.comments {
        metadata.add_frame(Comment {
            lang: String::from("eng"),
            description: String::new(),
            text: comments.clone(),
        });
    }

    for chapter in chapters {
        metadata.add_frame(chapter);
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args: Args = Args::parse();
    anyhow::ensure!(!args.files.is_empty(), "no input files specified");

    let chapters = get_chapters(&args).context("failed to generate chapter metadata")?;
    create_mergelist(&args).context("failed to create temporary mergelist")?;
    let merged_file = merge_files().context("failed to merge input files")?;

    let mut metadata = Tag::read_from_path(merged_file.path())
        .context("failed to read ID3 tag from merged file")?;

    populate_metadata(&args, &mut metadata, chapters).context("failed to set ID3 metadata")?;

    metadata
        .write_to_path(merged_file.path(), Version::Id3v24)
        .context("failed to write ID3 metadata to merged file")?;

    args.output.set_extension("mp3");
    fs::copy(merged_file.path(), &args.output).with_context(|| {
        format!(
            "failed to copy merged file to output path '{}'",
            args.output.to_string_lossy()
        )
    })?;

    Ok(())
}
