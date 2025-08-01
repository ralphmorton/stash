use std::str::FromStr;

use stash::{Client, File, Response, Tag};
use util::{ClientServer, TestInfra};

mod util;

#[tokio::test]
async fn file_cas() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra).await;
    let client = client_server.client;

    let tag1 = Tag::from_str("t1").unwrap();
    let tag2 = Tag::from_str("t2").unwrap();
    let tag3 = Tag::from_str("t3").unwrap();
    let tag4 = Tag::from_str("t4").unwrap();

    let file1 = create_file(
        &client,
        "f1",
        vec![tag1.clone(), tag2.clone()],
        false,
        b"hello",
    )
    .await
    .unwrap();

    let file2 = create_file(
        &client,
        "f2",
        vec![tag2.clone(), tag3.clone()],
        false,
        b"world",
    )
    .await
    .unwrap();

    let file3 = create_file(
        &client,
        "f3",
        vec![tag3.clone(), tag4.clone()],
        false,
        b"hello",
    )
    .await
    .unwrap();

    let file4 = create_file(
        &client,
        "f3",
        vec![tag3.clone(), tag4.clone()],
        false,
        b"foo",
    )
    .await;

    assert!(matches!(file4, Response::Err(_)));
    assert_eq!(file4.err(), "File already exists");

    assert_eq!(&file1.size, &file3.size);
    assert_eq!(&file1.hash, &file3.hash);

    let desc1 = client.describe(file1.name.clone()).await.unwrap().unwrap();
    let expected_tags: Vec<String> = vec![tag1.clone().into(), tag2.clone().into()];
    assert_eq!(desc1.tags, expected_tags);

    let desc2 = client.describe(file2.name.clone()).await.unwrap().unwrap();
    let expected_tags: Vec<String> = vec![tag2.clone().into(), tag3.clone().into()];
    assert_eq!(desc2.tags, expected_tags);

    let desc3 = client.describe(file3.name.clone()).await.unwrap().unwrap();
    let expected_tags: Vec<String> = vec![tag3.clone().into(), tag4.clone().into()];
    assert_eq!(desc3.tags, expected_tags);

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

    let mut files = client_server.infra.files().await;
    files.sort();
    assert_eq!(files, vec![file1.hash.clone(), file2.hash.clone()]);

    client.delete(file1.name.clone()).await.unwrap().unwrap();

    let mut files = client_server.infra.files().await;
    files.sort();
    assert_eq!(files, vec![file1.hash, file2.hash.clone()]);

    let tags1 = client.describe(file1.name.clone()).await.unwrap();
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

    let tag1 = Tag::from_str("t1").unwrap();
    let tag2 = Tag::from_str("t2").unwrap();
    let tag3 = Tag::from_str("t3").unwrap();

    let file1 = create_file(&client, "dir1/f1", vec![tag1.clone()], false, b"hello")
        .await
        .unwrap();

    let file2 = create_file(&client, "dir1/f2", vec![tag2.clone()], false, b"world")
        .await
        .unwrap();

    let file3 = create_file(
        &client,
        "dir2/f3",
        vec![tag1.clone(), tag3.clone()],
        false,
        b"hello",
    )
    .await
    .unwrap();

    let fx1 = client.list(tag1.clone(), None).await.unwrap().unwrap();
    assert_eq!(fx1, vec![file1.clone(), file3.clone()]);

    let fx1 = client
        .list(tag1.clone(), Some("dir1/".to_string()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fx1, vec![file1.clone()]);

    let fx2 = client.list(tag2.clone(), None).await.unwrap().unwrap();
    assert_eq!(fx2, vec![file2.clone()]);

    let fx3 = client.list(tag3.clone(), None).await.unwrap().unwrap();
    assert_eq!(fx3, vec![file3.clone()]);

    let sx1 = client
        .search(tag1.clone(), "f".to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(sx1, vec![file1.clone(), file3.clone()]);

    let sx1 = client
        .search(tag1.clone(), "f3".to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(sx1, vec![file3.clone()]);
}

#[tokio::test]
async fn file_replace() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra).await;
    let client = client_server.client;

    let tag = Tag::from_str("test").unwrap();

    let file1 = create_file(&client, "hello-1", vec![tag.clone()], false, b"hello")
        .await
        .unwrap();

    let file2 = create_file(&client, "hello-2", vec![tag.clone()], false, b"hello")
        .await
        .unwrap();

    let files = client.list(tag.clone(), None).await.unwrap().res().unwrap();
    assert_eq!(files, vec![file1.clone(), file2.clone()]);

    let mut files = client_server.infra.files().await;
    files.sort();
    assert_eq!(files, vec![file1.hash.clone()]);

    let file3 = create_file(&client, "hello-1", vec![tag.clone()], false, b"world").await;
    assert!(matches!(file3, Response::Err(_)));
    assert_eq!(file3.err(), "File already exists");

    let file3 = create_file(&client, "hello-1", vec![tag.clone()], true, b"world")
        .await
        .unwrap();

    let mut files = client_server.infra.files().await;
    files.sort();
    assert_eq!(files, vec![file1.hash.clone(), file3.hash.clone()]);

    let files = client.list(tag, None).await.unwrap().res().unwrap();
    assert_eq!(files, vec![file3, file2]);
}

async fn create_file(
    client: &Client,
    name: &str,
    tags: Vec<Tag>,
    replace: bool,
    content: &[u8],
) -> Response<File> {
    let blob = client.create_blob().await.unwrap().unwrap();
    let blob = client
        .append_blob(blob.name, content.to_vec())
        .await
        .unwrap()
        .unwrap();

    client
        .commit_blob(blob.name, name.to_string(), tags, replace)
        .await
        .unwrap()
}
