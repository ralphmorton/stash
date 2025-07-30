use std::path::PathBuf;

use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;

use crate::{Error, SHA256};

pub async fn digest(path: &PathBuf) -> Result<SHA256, Error> {
    let mut hasher = Sha256::new();
    let mut file = tokio::fs::File::open(path).await?;
    let mut buf = [0u8; 10_000];

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        hasher.update(&buf[0..n]);
    }

    let hash = hasher.finalize();
    let hash = data_encoding::HEXLOWER.encode(&hash);
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn sha256_file_digests() {
        let mut f1 = NamedTempFile::new().unwrap();
        let mut f2 = NamedTempFile::new().unwrap();
        let mut f3 = NamedTempFile::new().unwrap();

        f1.write_all(b"test1").unwrap();
        f2.write_all(b"test2").unwrap();
        f3.write_all(b"test1").unwrap();

        let h1 = super::digest(&f1.path().to_path_buf()).await.unwrap();
        let h2 = super::digest(&f2.path().to_path_buf()).await.unwrap();
        let h3 = super::digest(&f3.path().to_path_buf()).await.unwrap();

        assert!(h1 != h2);
        assert!(h1 == h3);
    }
}
