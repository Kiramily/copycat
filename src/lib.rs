use std::{
    error::Error,
    fs::{self, create_dir_all, read_dir},
    io::{self, ErrorKind},
    path::Path,
};

use rayon::ThreadPoolBuilder;

pub fn copy<P: AsRef<Path>>(from: P, to: P, threads: Option<usize>) -> Result<(), Box<dyn Error>> {
    let threads = threads.unwrap_or(num_cpus::get());

    let pool = ThreadPoolBuilder::new().num_threads(threads).build()?;

    let from = from.as_ref();
    let to = to.as_ref();

    pool.install(|| walk(from, to).unwrap());
    Ok(())
}

fn walk(start: &Path, destination: &Path) -> io::Result<()> {
    if !start.exists() || !start.is_dir() {
        return Ok(());
    }

    if !destination.exists() {
        create_dir_all(destination).expect("Could not create destination directory");
    }

    rayon::scope(|scope| {
        for entry in read_dir(start).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let destination_path = destination.join(path.file_name().unwrap());

            if path.is_file() {
                scope.spawn(move |_| copy_file(&path, &destination_path));
            } else if path.is_dir() {
                scope.spawn(move |_| walk(&path, &destination_path).unwrap());
            }
        }
    });

    Ok(())
}

fn copy_file(file: &Path, destination: &Path) {
    if destination.exists() {
        // Check Modification Date

        let dest_meta = fs::metadata(destination).unwrap();
        let file_meta = fs::metadata(file).unwrap();

        if dest_meta.modified().unwrap() == file_meta.modified().unwrap() {
            return;
        } else {
            if dest_meta.len() == file_meta.len() {
                // Do nothing when the files are the same size
                return;
            }

            println!(
                "removing {} because of unmatched modified dates and different file sizes",
                destination.display()
            );
            fs::remove_file(destination).unwrap_or_else(|_| {
                panic!(
                    "Failed to remove file: {} with unmatched modified dates",
                    destination.display()
                )
            });
        }
    }

    println!("Copying file: {}", file.display());

    match fs::copy(file, destination) {
        Ok(_) => {}
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => eprintln!(
                "Permission Denied to copy file \"{}\" to \"{}\"",
                file.display(),
                destination.display()
            ),
            ErrorKind::AlreadyExists => eprintln!(
                "The destination \"{}\" already exists",
                destination.display()
            ),
            _ => eprintln!("Error while copying: {e}"),
        },
    }
}
