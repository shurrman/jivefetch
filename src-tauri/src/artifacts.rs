use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::storage::RemovalTarget;

const PARTIAL_ROOT: &str = ".jivefetch-partials";

pub fn partial_directory(output_directory: &Path, artifact_key: &str) -> io::Result<PathBuf> {
    if artifact_key.len() != 32 || !artifact_key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "artifact key is not a 32-character hexadecimal identifier",
        ));
    }
    Ok(output_directory.join(PARTIAL_ROOT).join(artifact_key))
}

pub fn remove_task_files(output_directory: &Path, target: &RemovalTarget) -> io::Result<usize> {
    let canonical_output = output_directory.canonicalize()?;
    let mut removed = 0;

    if !target.output_path_shared {
        if let Some(output_path) = target.output_path.as_deref() {
            removed += remove_tracked_file(Path::new(output_path), &canonical_output)?;
        }
    }

    if let Some(artifact_key) = target.artifact_key.as_deref() {
        let partial_directory = partial_directory(output_directory, artifact_key)?;
        removed += remove_partial_directory(&partial_directory, &canonical_output, artifact_key)?;
    }

    Ok(removed)
}

fn remove_tracked_file(path: &Path, canonical_output: &Path) -> io::Result<usize> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "tracked output is not a regular file",
        ));
    }
    let canonical_path = path.canonicalize()?;
    canonical_path.strip_prefix(canonical_output).map_err(|_| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "tracked output is outside the configured directory",
        )
    })?;
    fs::remove_file(canonical_path)?;
    Ok(1)
}

fn remove_partial_directory(
    path: &Path,
    canonical_output: &Path,
    artifact_key: &str,
) -> io::Result<usize> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "task partial path is not a directory",
        ));
    }
    let canonical_path = path.canonicalize()?;
    let expected_relative = Path::new(PARTIAL_ROOT).join(artifact_key);
    if canonical_path.strip_prefix(canonical_output).ok() != Some(expected_relative.as_path()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "task partial directory is outside its owned location",
        ));
    }
    fs::remove_dir_all(canonical_path)?;
    Ok(1)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{partial_directory, remove_task_files};
    use crate::storage::RemovalTarget;

    const KEY: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn removes_only_the_tracked_output_and_owned_partial_directory() {
        let output = tempfile::tempdir().unwrap();
        let final_file = output.path().join("video.mp4");
        let unrelated = output.path().join("other.mp4");
        fs::write(&final_file, b"video").unwrap();
        fs::write(&unrelated, b"other").unwrap();
        let partials = partial_directory(output.path(), KEY).unwrap();
        fs::create_dir_all(partials.join("fragments")).unwrap();
        fs::write(partials.join("video.part"), b"partial").unwrap();
        fs::write(partials.join("fragments/one.part"), b"fragment").unwrap();

        let removed = remove_task_files(
            output.path(),
            &RemovalTarget {
                output_path: Some(final_file.to_string_lossy().into_owned()),
                artifact_key: Some(KEY.to_string()),
                artifact_root: Some(output.path().to_string_lossy().into_owned()),
                output_path_shared: false,
            },
        )
        .unwrap();

        assert_eq!(removed, 2);
        assert!(!final_file.exists());
        assert!(!partials.exists());
        assert!(unrelated.exists());
    }

    #[test]
    fn refuses_an_output_outside_the_configured_directory() {
        let output = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        let result = remove_task_files(
            output.path(),
            &RemovalTarget {
                output_path: Some(outside.path().to_string_lossy().into_owned()),
                artifact_key: None,
                artifact_root: None,
                output_path_shared: false,
            },
        );

        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert!(outside.path().exists());
    }

    #[test]
    fn preserves_a_final_file_that_is_shared_by_another_task() {
        let output = tempfile::tempdir().unwrap();
        let final_file = output.path().join("shared.mp4");
        fs::write(&final_file, b"video").unwrap();

        let removed = remove_task_files(
            output.path(),
            &RemovalTarget {
                output_path: Some(final_file.to_string_lossy().into_owned()),
                artifact_key: None,
                artifact_root: None,
                output_path_shared: true,
            },
        )
        .unwrap();

        assert_eq!(removed, 0);
        assert!(final_file.exists());
    }
}
