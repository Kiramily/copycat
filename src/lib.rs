use rayon::ThreadPoolBuilder;
use std::{
    fs::{self, create_dir_all, read_dir, File, FileTimes},
    io::{self, ErrorKind},
    path::Path,
};
use tracing::{debug, error, info};

#[derive(Clone, Copy)]
pub enum Comparison {
    None,
    Exists,
    Date,
}

pub fn copy<P: AsRef<Path>>(
    from: P,
    to: P,
    follow_links: bool,
    comparison: Comparison,
    copy_metadata: bool,
    threads: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let threads = threads.unwrap_or(num_cpus::get());

    let pool = ThreadPoolBuilder::new().num_threads(threads).build()?;

    let from = from.as_ref();
    let to = to.as_ref();

    pool.install(|| walk(from, to, follow_links, comparison, copy_metadata))?;

    Ok(())
}

fn walk(
    start: &Path,
    destination: &Path,
    follow_links: bool,
    comparison: Comparison,
    copy_metadata: bool,
) -> io::Result<()> {
    if !start.exists() || !start.is_dir() {
        return Ok(());
    }

    if !destination.exists() {
        create_dir_all(destination)?;
    }

    rayon::scope(|scope| {
        for entry in read_dir(start).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let destination_path = destination.join(path.file_name().unwrap());

            if path.is_file() {
                scope
                    .spawn(move |_| copy_file(&path, &destination_path, comparison, copy_metadata));
            } else if path.is_dir() {
                scope.spawn(move |_| {
                    walk(
                        &path,
                        &destination_path,
                        follow_links,
                        comparison,
                        copy_metadata,
                    )
                    .unwrap()
                });
            } else if path.is_symlink() && follow_links {
                match fs::read_link(path) {
                    Ok(p) => scope.spawn(move |_| {
                        walk(
                            &p,
                            &destination_path,
                            follow_links,
                            comparison,
                            copy_metadata,
                        )
                        .unwrap()
                    }),
                    Err(error) => tracing::error!(?error, "failed to read symlink"),
                };
            }
        }
    });

    Ok(())
}

fn copy_file(file: &Path, destination: &Path, comparison: Comparison, copy_metadata: bool) {
    if destination.exists() {
        match comparison {
            Comparison::None => {}
            Comparison::Exists => return,
            Comparison::Date => {
                let file_meta = fs::metadata(file).unwrap();

                let dest_meta = match fs::metadata(destination) {
                    Ok(m) => m,
                    Err(error) => {
                        error!(?error, file = ?destination, "failed to get metadata");
                        return;
                    }
                };

                if dest_meta
                    .modified()
                    .map_or(false, |d| file_meta.modified().map_or(false, |f| f == d))
                {
                    debug!(source = ?file, destination = ?destination, "skipping matching files");
                    return;
                }

                if dest_meta.permissions().readonly() {
                    error!(file = ?destination, "target file is readonly");

                    return;
                }
            }
        }
    }

    info!(source = ?file, target = ?destination, "copying file");

    match fs::copy(file, destination) {
        Ok(_) => {
            if copy_metadata {
                match copy_filetimes(file, destination) {
                    Ok(_) => debug!(source = ?file, target = ?destination, "copied metadata"),
                    Err(error) => {
                        error!(?error, source = ?file, target = ?destination, "failed to copy metadata. your current platform may not support accessing or modifying file modification and access dates.")
                    }
                }
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => {
                error!(source = ?file, target = ?destination, "Permission Denied to copy file")
            }
            ErrorKind::AlreadyExists => {
                error!(source = ?file, target = ?destination, "The destination already exists")
            }
            _ => {
                error!(error = ?e, source = ?file, target = ?destination, "An Error occurred while copying")
            }
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
