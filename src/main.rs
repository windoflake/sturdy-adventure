
use std::fs;
use clap::{Parser};

type Result<T> = std::result::Result<T, std::io::Error>;

#[derive(Debug, Parser)]
#[clap(name = "batch_renamer",
          version = "0.1",
          author = "Ross Yang",
          about = "This is a renaming tool to rename all files inside a folder.",
          )]
struct Args {
    #[arg(short, long, default_value = ".", help = "The directory to rename files in")]
    directory: Option<String>,
    #[arg(
        short,
        long,
        value_parser,
        num_args = 1..,
        default_value = "mp3",
        value_delimiter = ',',
        help = "Only files ends with the given extensions are to be renamed",)]
    extensions: Vec<String>,
    #[arg(short,
          long,
          value_parser,
          num_args = 1..2,
          value_delimiter = ',',
          default_value = "-",
          help = "The separator to use, e.g. `-` or `.`. But `,` is not allowed. At most two separators are allowed. The first separator is used to split the file name into two parts, and the second separator is used to join the two parts back together.")]
    separator: Vec<String>,
    #[arg(short, long, default_value = "", help = "The padding to use")]
    padding: String,
    #[arg(short, long, default_value_t = false, help = "Whether to rename files recursively")]
    recursive: bool,
}

/**
 * Renames the files inside the given directory with the specified extensions,
 * separators, padding, and recursion flag. Returns the number of files renamed.
 *
 * @param directory The directory in which to rename files.
 * @param extensions A list of file extensions to consider for renaming.
 * @param old_sep Separator to split the file name into two parts.
 * @param new_sep Separator to join the two parts back together.
 * @param padding Padding string to use between the separated parts of the new file name.
 * @param recursive Whether to rename files recursively in subdirectories.
 *
 * @return The number of files renamed.
 * @throws std::io::Error if file renaming encounters any issues.
 */
fn rename_files_swapped(directory: &str, extensions: &[&str],
                        old_sep: &str, new_sep: &str,
                        padding: &str, recursive: bool) -> Result<u64> {
    let paths = fs::read_dir(directory)?;
    let mut files_renamed = 0;

    for path in paths {
        let path = path?.path();
        if path.is_dir() {
            if recursive {
                files_renamed += rename_files_swapped(
                    path.to_str().unwrap(), extensions, old_sep, new_sep, padding, recursive)?;
            }
            continue;
        }

        if let Some(extension) = path.extension() {
            let extension = extension.to_str().expect("Could not get extension");

            if extensions.contains(&extension) {
                let file_stem = path.file_stem().unwrap().to_str().unwrap();
                let filenames = file_stem
                    .rsplit(&old_sep)
                    .map(|s| s.trim())
                    .collect::<Vec<&str>>();

                let old_path = path.to_str().unwrap();
                if filenames.len() != 2 {
                    println!("Skipping `{}`", old_path);
                    continue;
                }
                
                let separator = format!("{}{}{}", padding, new_sep, padding);
                let mut new_file_name = filenames
                    .join(&separator);

                let extension = format!(".{}", extension);
                new_file_name.push_str(&extension);

                let new_path = path.parent().unwrap().join(new_file_name);
                let new_path = new_path.to_str().unwrap();
                println!("Renaming `{}` to `{}`", old_path, new_path);

                fs::rename(path, new_path)?;

                files_renamed += 1;
            }
        } 
    }

    Ok(files_renamed)
}

fn main() {
    let args = Args::parse();
    println!("We are renaming files in folder {:?} with extensions {:?} ... ",
        args.directory.as_ref().unwrap(), args.extensions);

    let directory = args.directory.unwrap();
    let extensions = args.extensions.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    let separator = args.separator.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    let padding = args.padding.as_str();
    let recursive = args.recursive;

    // unescape the separator if it is escaped, since dash, which is a special character, could be escaped
    let old_sep = separator[0].replace("\\", "");
    let old_sep = old_sep.as_ref();
    let new_sep = if separator.len() > 1 { separator[1] } else { old_sep };
    let renamed = rename_files_swapped(&directory, &extensions, old_sep, new_sep, &padding, recursive)
        .expect("Could not rename files");

    if renamed == 0 {
        println!("Oops! No files were renamed.");
    } else {
        println!("Renamed {} files.", renamed);
    }
}
