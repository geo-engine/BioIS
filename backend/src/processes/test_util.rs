use crate::processes::parameters::DataResource;
use serde::Serialize;
use std::{io::Write, process::Command};
use tempfile::NamedTempFile;

/// Assert that the given data resource is valid.
/// It does so by writing it to a temporary file and checking with `frictionless` using `pipx`.
///
/// # Panics
///
/// Panics if the data resource is not valid.
///
pub fn assert_valid_data_resource<T>(data_resource: impl AsRef<DataResource<T>>)
where
    T: Serialize,
{
    let mut tempfile = NamedTempFile::with_suffix("-test-data-resource.json")
        .expect("Failed to create temporary file");
    let json = serde_json::to_string(data_resource.as_ref())
        .expect("Failed to serialize data resource to JSON");
    write!(&mut tempfile, "{json}").expect("Failed to write JSON to temporary file");
    tempfile.flush().expect("Failed to flush temporary file");

    let output = Command::new("pipx")
        .args([
            "run",
            "--spec",
            "git+https://github.com/frictionlessdata/frictionless-py.git", // TODO: use `PyPI` version once new version is released
            "frictionless",
            "validate",
            tempfile
                .path()
                .to_str()
                .expect("Failed to convert tempfile path to string"),
        ])
        .output()
        .expect("Failed to execute pipx");

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "Validation failed:\n{stderr}");
}
