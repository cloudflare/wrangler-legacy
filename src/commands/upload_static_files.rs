use walkdir::WalkDir;
use percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};

use crate::settings::global_user::GlobalUser;

use std::ffi::OsString;


pub fn upload_static_files(user: &GlobalUser, namespace: &str, directory: &str) {
    println!("Uploading {} to {}", directory, namespace);

    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();

        // directories don't have extensions, and we don't want to upload them anyway, so 
        // let's only do this for files
        if entry.path().is_file() {
            // okay, now comes some shenanigans.

            // first, we strip the base name off; we don't want that in the URL
            let path = entry.path().strip_prefix(directory).unwrap();

            // next, we have to re-build the paths: if we're on Windows, we have paths with
            // `\` as separators. But we want to use `/` as separators. Because that's how URLs
            // work.
            let mut path_with_forward_slash = OsString::new();

            for (i, component) in path.components().enumerate() {
                // we don't want a leading `/`, so skip that
                if i > 0 {
                    path_with_forward_slash.push("/");
                }

                path_with_forward_slash.push(component);
            }

            // if we have a non-utf8 path here, it will fail, but that's not realistically going to happen
            let bytes = path_with_forward_slash.to_str().expect("found a non-UTF-8 path").as_bytes();

            let encoded_filename = percent_encode(bytes, DEFAULT_ENCODE_SET);

            println!("{}", encoded_filename.to_string());
        }
    }
}