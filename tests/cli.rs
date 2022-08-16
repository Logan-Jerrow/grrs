use assert_cmd::Command;
use assert_fs::{
    prelude::{FileWriteBin, FileWriteStr},
    NamedTempFile,
};
use predicates::prelude::PredicateBooleanExt;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const NAME: &str = env!("CARGO_PKG_NAME");

#[test]
fn file_doesnt_exist() -> TestResult {
    let mut cmd = Command::cargo_bin(NAME)?;
    cmd.arg("foo").arg("does/not/exist.txt");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("path does not exists"));

    Ok(())
}

#[test]
fn no_match() -> TestResult {
    let temp = NamedTempFile::new("temp_test.txt")?;
    temp.write_str("A test\nActual content\nMore content\nAnother test")?;

    let mut cmd = Command::cargo_bin(NAME)?;
    cmd.arg("foobar").arg(temp.path());
    cmd.assert().success().stdout(predicates::str::is_empty());

    Ok(())
}

#[test]
fn empty_pattern() -> TestResult {
    let temp = NamedTempFile::new("temp_test.txt")?;

    let mut cmd = Command::cargo_bin(NAME)?;
    cmd.arg("").arg(temp.path());
    cmd.assert().failure().stderr(predicates::str::contains(
        "error: The argument '<PATTERN>' requires a value but none was supplied",
    ));

    Ok(())
}

#[test]
fn empty_path() -> TestResult {
    let mut cmd = Command::cargo_bin(NAME)?;
    cmd.arg("foo").arg("");
    cmd.assert().failure().stderr(predicates::str::contains(
        "error: The argument '<PATH>' requires a value but none was supplied",
    ));

    Ok(())
}

#[test]
fn file_with_non_utf8() -> TestResult {
    let temp = NamedTempFile::new("temp_test.txt")?;
    let invalid = b"\xE0\x80\x80\n\xED\xBF\xBF\n\xF0\x80\x80\x80\n\xF4\xBF\xBF\xBF";
    temp.write_binary(invalid)?;

    let mut cmd = Command::cargo_bin(NAME)?;
    cmd.arg("foobar").arg(temp.path());
    cmd.assert().failure().stderr(
        predicates::str::contains("Error: error while reading file:").and(
            predicates::str::contains("stream did not contain valid UTF-8"),
        ),
    );

    Ok(())
}

#[test]
fn find_contents_in_file() -> TestResult {
    let temp = NamedTempFile::new("temp_test.txt")?;
    temp.write_str("A test\nActual content\nMore content\nAnother test")?;

    let mut cmd = Command::cargo_bin(NAME)?;
    cmd.arg("test").arg(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("A test\nAnother test"));

    Ok(())
}
