//! Tokio-based async common I/O code.

use async_compression::tokio::bufread::GzipDecoder;
use std::path::Path;
use std::pin::Pin;
use tokio::fs::File;
use tokio::io::{AsyncRead, BufReader};

use crate::common::io::std::is_gz;

/// Transparently open a file with gzip decoder.
///
/// # Arguments
///
/// * `path` - A path to the file to open.
///
/// # Returns
///
/// A `Result` containing a `Pin<Box<dyn AsyncRead>>` that can be used to read
/// the file, or an `anyhow::Error` if the file could not be opened.
pub async fn open_read_maybe_gz<P>(path: P) -> Result<Pin<Box<dyn AsyncRead>>, anyhow::Error>
where
    P: AsRef<Path>,
{
    tracing::trace!(
        "Opening {} as {} reading",
        path.as_ref().display(),
        "palin text"
    );
    let file = File::open(path.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("could not open file {}: {}", path.as_ref().display(), e))?;

    if is_gz(path.as_ref()) {
        let bufreader = BufReader::new(file);
        let decoder = {
            let mut decoder = GzipDecoder::new(bufreader);
            decoder.multiple_members(true);
            decoder
        };
        Ok(Box::pin(decoder))
    } else {
        Ok(Box::pin(BufReader::new(file)))
    }
}

#[cfg(test)]
mod test {
    use tokio::io::AsyncReadExt;

    #[rstest::rstest]
    #[case("14kb.txt")]
    #[case("14kb.txt.gz")]
    #[case("14kb.txt.bgz")]
    #[tokio::test]
    async fn open_read_maybe_gz(#[case] path: &str) -> Result<(), anyhow::Error> {
        mehari::common::set_snapshot_suffix!("{}", path);
        // Note that the 14kb.txt file contains about 14 KB of data so bgz will have multiple 4KB
        // blocks.

        let mut reader = super::open_read_maybe_gz(&format!("tests/common/io/{}", path)).await?;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        insta::assert_snapshot!(String::from_utf8(buf)?);

        Ok(())
    }
}
