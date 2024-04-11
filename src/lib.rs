use rayon::ThreadPoolBuilder;
use std::{
    fs::{self, create_dir_all, read_dir, File, FileTimes},
    io::{self, ErrorKind},
    path::Path,
    time::{Duration, SystemTime},
};
use tracing::{debug, error, info};

pub fn copy<P: AsRef<Path>>(
    from: P,
    to: P,
    threads: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
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
            debug!(?destination, "removing because of unmatched modified dates");

            fs::remove_file(destination).unwrap_or_else(|_| {
                panic!(
                    "Failed to remove file: {} with unmatched modified dates",
                    destination.display()
                )
            });
        }
    }

    info!(?file, "Copying file");

    match fs::copy(file, destination) {
        Ok(_) => {
            copy_filetimes(file, destination).unwrap();
        }
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => {
                error!(?file, ?destination, "Permission Denied to copy file")
            }
            ErrorKind::AlreadyExists => error!(?destination, "The destination already exists"),
            _ => error!(?e, "An Error occurred while copying"),
        },
    }
}

fn copy_filetimes<F: AsRef<Path>, T: AsRef<Path>>(from: F, to: T) -> io::Result<()> {
    let metadata = fs::metadata(from)?;
    let file = File::open(to)?;

    let times = FileTimes::new()
        .set_accessed(metadata.accessed()?)
        .set_modified(metadata.modified()?);

    file.set_times(times)?;

    drop(file);

    Ok(())
}
