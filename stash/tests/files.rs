use std::str::FromStr;

use stash::{Response, Tag};
use util::{ClientServer, TestInfra};

mod util;

#[tokio::test]
async fn file_management() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra).await;
    let client = client_server.client;

    let blob1 = client.create_blob().await.unwrap().unwrap();
    let blob1 = client
        .append_blob(blob1.name, b"hello".to_vec())
        .await
        .unwrap()
        .unwrap();

    let blob2 = client.create_blob().await.unwrap().unwrap();
    let blob2 = client
        .append_blob(blob2.name, b"world".to_vec())
        .await
        .unwrap()
        .unwrap();

    let blob3 = client.create_blob().await.unwrap().unwrap();
    let blob3 = client
        .append_blob(blob3.name, b"hello".to_vec())
        .await
        .unwrap()
        .unwrap();

    let blob4 = client.create_blob().await.unwrap().unwrap();
    let blob4 = client
        .append_blob(blob4.name, b"foo".to_vec())
        .await
        .unwrap()
        .unwrap();

    let file1 = client
        .commit_blob(
            blob1.name,
            "f1".to_string(),
            vec![Tag::from_str("t1").unwrap(), Tag::from_str("t2").unwrap()],
        )
        .await
        .unwrap()
        .unwrap();

    let file2 = client
        .commit_blob(
            blob2.name,
            "f2".to_string(),
            vec![Tag::from_str("t2").unwrap(), Tag::from_str("t3").unwrap()],
        )
        .await
        .unwrap()
        .unwrap();

    let file3 = client
        .commit_blob(
            blob3.name,
            "f3".to_string(),
            vec![Tag::from_str("t3").unwrap(), Tag::from_str("t4").unwrap()],
        )
        .await
        .unwrap()
        .unwrap();

    let file4 = client
        .commit_blob(
            blob4.name.clone(),
            "f3".to_string(),
            vec![Tag::from_str("t4").unwrap(), Tag::from_str("t5").unwrap()],
        )
        .await
        .unwrap();
    assert!(matches!(file4, Response::Err(_)));
    assert_eq!(file4.err(), "File already exists");

    assert_eq!(&file1.size, &file3.size);
    assert_eq!(&file1.hash, &file3.hash);

    let tags1 = client.tags(file1.name.clone()).await.unwrap().unwrap();
    assert_eq!(tags1, vec!["t1".to_string(), "t2".to_string()]);

    let tags2 = client.tags(file2.name.clone()).await.unwrap().unwrap();
    assert_eq!(tags2, vec!["t2".to_string(), "t3".to_string()]);

    let tags3 = client.tags(file3.name.clone()).await.unwrap().unwrap();
    assert_eq!(tags3, vec!["t3".to_string(), "t4".to_string()]);

    let data1 = client
        .download(file1.hash.clone(), 0, 5)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(data1, b"hello".to_vec());

    let data2 = client
        .download(file2.hash.clone(), 0, 5)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(data2, b"world".to_vec());

    let data3 = client.download(file3.hash, 0, 5).await.unwrap().unwrap();
    assert_eq!(data3, b"hello".to_vec());

    let blobs = client_server.infra.blobs().await;
    assert_eq!(blobs, vec![blob4.name]);

    let mut files = client_server.infra.files().await;
    files.sort();
    assert_eq!(files, vec![file1.hash.clone(), file2.hash.clone()]);

    client.delete(file1.name.clone()).await.unwrap().unwrap();

    let mut files = client_server.infra.files().await;
    files.sort();
    assert_eq!(files, vec![file1.hash, file2.hash.clone()]);

    let tags1 = client.tags(file1.name.clone()).await.unwrap();
    assert!(matches!(tags1, Response::Err(_)));
    assert_eq!(tags1.err(), "No such file");

    client.delete(file3.name.clone()).await.unwrap().unwrap();

    let mut files = client_server.infra.files().await;
    files.sort();
    assert_eq!(files, vec![file2.hash]);
}

#[tokio::test]
async fn file_lookup() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra).await;
    let client = client_server.client;

    let blob1 = client.create_blob().await.unwrap().unwrap();
    let blob1 = client
        .append_blob(blob1.name, b"hello".to_vec())
        .await
        .unwrap()
        .unwrap();

    let blob2 = client.create_blob().await.unwrap().unwrap();
    let blob2 = client
        .append_blob(blob2.name, b"world".to_vec())
        .await
        .unwrap()
        .unwrap();

    let blob3 = client.create_blob().await.unwrap().unwrap();
    let blob3 = client
        .append_blob(blob3.name, b"hello".to_vec())
        .await
        .unwrap()
        .unwrap();

    let file1 = client
        .commit_blob(
            blob1.name,
            "dir1/f1".to_string(),
            vec![Tag::from_str("t1").unwrap()],
        )
        .await
        .unwrap()
        .unwrap();

    let file2 = client
        .commit_blob(
            blob2.name,
            "dir1/f2".to_string(),
            vec![Tag::from_str("t2").unwrap()],
        )
        .await
        .unwrap()
        .unwrap();

    let file3 = client
        .commit_blob(
            blob3.name,
            "dir2/f3".to_string(),
            vec![Tag::from_str("t1").unwrap(), Tag::from_str("t3").unwrap()],
        )
        .await
        .unwrap()
        .unwrap();

    let fx1 = client
        .list(Tag::from_str("t1").unwrap(), None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fx1, vec![file1.clone(), file3.clone()]);

    let fx1 = client
        .list(Tag::from_str("t1").unwrap(), Some("dir1/".to_string()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fx1, vec![file1.clone()]);

    let fx2 = client
        .list(Tag::from_str("t2").unwrap(), None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fx2, vec![file2.clone()]);

    let fx3 = client
        .list(Tag::from_str("t3").unwrap(), None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fx3, vec![file3.clone()]);

    let sx1 = client
        .search(Tag::from_str("t1").unwrap(), "f".to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(sx1, vec![file1.clone(), file3.clone()]);

    let sx1 = client
        .search(Tag::from_str("t1").unwrap(), "f3".to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(sx1, vec![file3.clone()]);
}
