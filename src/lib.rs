use std::{
    error::Error,
    fs::{self, create_dir_all, read_dir},
    io,
    path::Path,
};

use rayon::ThreadPoolBuilder;

pub fn copy<P: AsRef<Path>>(from: P, to: P, threads: Option<usize>) -> Result<(), Box<dyn Error>> {
    let threads = threads.unwrap_or(num_cpus::get());

    assert!(threads <= num_cpus::get(), "?????");

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
        create_dir_all(&destination).unwrap();
    }

    rayon::scope(|scope| {
        for entry in read_dir(start).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() {
                let destination_path = destination.join(path.file_name().unwrap());
                scope.spawn(move |_| copy_file(&path, &destination_path));
            } else if path.is_dir() {
                let destination_path = destination.join(path.file_name().unwrap());
                scope.spawn(move |_| walk(&path, &destination_path).unwrap());
            }
        }
    });

    Ok(())
}

fn copy_file(file: &Path, destination: &Path) {
    if destination.exists() {
        // Check Modification Date

        let dest_meta = fs::metadata(&destination).unwrap();
        let file_meta = fs::metadata(&file).unwrap();

        if dest_meta.modified().unwrap() == file_meta.modified().unwrap() {
            return;
        }
    }

    println!("Copying file: {}", file.display());

    fs::copy(&file, &destination).unwrap();
}
