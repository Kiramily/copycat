#![allow(clippy::not_unsafe_ptr_arg_deref)]
use log::{debug, info, warn};
use rayon::ThreadPoolBuilder;
use std::{
    ffi::CStr,
    fs::{self, create_dir_all, read_dir},
    os::raw::c_char,
    path::{Path, PathBuf},
};

bitflags::bitflags! {
    #[repr(C)]
    pub struct CopyFlags: u32 {
        const NONE = 1 << 0;
        const OVERWRITE = 1 << 1;
        const RECURSIVE = 1<<2;
        const SKIP_EXISTING =1<<3;
        const NO_OVERWRITE = 1<<4;
        const FOLLOW_SYMLINKS = 1<<5;
    }
}

/// Copy a file or directory from one location to another.
/// If the source is a directory, the destination must be a directory as well.
/// If the source is a file, the destination must be a file as well.
///
/// # Panics
/// Idk tbh lmao
///
#[no_mangle]
pub extern "C" fn cc_copy(
    source: *const c_char,
    destination: *const c_char,
    flags: CopyFlags,
    threads: usize,
) {
    let source = unsafe {
        assert!(!source.is_null());
        CStr::from_ptr(source)
    };
    let destination = unsafe {
        assert!(!destination.is_null());
        CStr::from_ptr(destination)
    };

    let source = PathBuf::from(source.to_str().unwrap());
    let destination = PathBuf::from(destination.to_str().unwrap());

    copy(&source, &destination, flags, threads);
}

/// Copy a file or directory from `source` to `destination`.
fn copy_file(source: &Path, destination: &Path, flags: CopyFlags) {
    // If the destination exists, and we're not overwriting, then we're done.
    if (flags.contains(CopyFlags::SKIP_EXISTING) || flags.contains(CopyFlags::NO_OVERWRITE))
        && destination.exists()
    {
        info!("Skipping existing file: {}", destination.display());
        return;
    }

    // If the destination exists, and we're overwriting, then remove it.
    if flags.contains(CopyFlags::OVERWRITE) {
        info!(
            "Deleting file: {} [Reason: Overwrite]",
            destination.display()
        );
        fs::remove_file(destination).unwrap();
    }

    // Copy the file.
    info!("Copying file: {}", source.display());
    fs::copy(source, destination).expect("Failed to copy file");
    info!("Copied file: {}", source.display());
}

fn do_copy(source: &Path, destination: &Path, flags: CopyFlags) {
    if !destination.exists() {
        warn!("Destination does not exist: {}", destination.display());
        debug!("Creating directory {}", destination.display());
        create_dir_all(destination).expect("Failed to create destination directory");
    }

    if source.is_file() {
        copy_file(source, destination, flags);
    } else if source.is_dir() {
        rayon::scope(|scope| {
            info!("Checking directory: {}", source.display());
            for entry in read_dir(source).expect("Failed to read source directory") {
                let entry = entry.expect("Failed to read source directory entry");
                let path = entry.path();
                let metadata = fs::metadata(&path).expect("Failed to get metadata");

                if metadata.is_dir() {
                    let destination_path = destination.join(path.file_name().unwrap());
                    scope.spawn(move |_| {
                        do_copy(&path, &destination_path, flags);
                    });
                } else if metadata.is_file() {
                    let destination_path = destination.join(path.file_name().unwrap());
                    scope.spawn(move |_| {
                        copy_file(&path, &destination_path, flags);
                    });
                } else if metadata.is_symlink() && flags.contains(CopyFlags::FOLLOW_SYMLINKS) {
                    // TODO: Follow symlinks.
                }
            }
        });
    }
}

/// Copy a file or directory from `source` to `destination`.
///
/// # Panics
/// Idk.
///
/// Panics if you're stupid.
pub fn copy(source: &Path, destination: &Path, flags: CopyFlags, threads: usize) {
    env_logger::init();

    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Failed to create thread pool");

    pool.install(|| do_copy(source, destination, flags));
}
