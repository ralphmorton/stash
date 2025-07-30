use std::str::FromStr;

use stash::{Response, Tag};
use util::{ClientServer, TestInfra};

mod util;

#[tokio::test]
async fn blob_management() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra).await;
    let client = client_server.client;

    let blob = client.create_blob().await.unwrap();
    assert!(matches!(blob, Response::Ok(_)));

    let blob = blob.unwrap();
    assert_eq!(blob.size, 0);

    let blob_name = &blob.name.to_string();

    let blob2 = client.describe_blob(blob_name.clone()).await.unwrap();
    assert!(matches!(blob2, Response::Ok(_)));
    assert_eq!(blob, blob2.unwrap());

    let blob = client
        .append_blob(blob_name.clone(), b"hello".to_vec())
        .await
        .unwrap();
    assert!(matches!(blob, Response::Ok(_)));

    let blob = blob.unwrap();
    assert_eq!(blob.size, 5);

    let blob = client
        .append_blob(blob_name.clone(), b" world".to_vec())
        .await
        .unwrap();
    assert!(matches!(blob, Response::Ok(_)));

    let blob = blob.unwrap();
    assert_eq!(blob.size, 11);

    let blob = client.describe_blob(blob_name.clone()).await.unwrap();
    assert!(matches!(blob, Response::Ok(_)));

    let blob = blob.unwrap();
    assert_eq!(blob.size, 11);

    let file = client
        .commit_blob(
            blob_name.clone(),
            "test-file".to_string(),
            vec![Tag::from_str("t1").unwrap(), Tag::from_str("t2").unwrap()],
        )
        .await
        .unwrap();
    assert!(matches!(file, Response::Ok(_)));

    let file = file.unwrap();
    assert_eq!(file.name, "test-file");
    assert_eq!(file.size, 11);

    let file_tags = client.tags(file.name).await.unwrap();
    assert!(matches!(file_tags, Response::Ok(_)));
    assert_eq!(file_tags.unwrap(), vec!["t1".to_string(), "t2".to_string()]);

    let rsp = client.describe_blob(blob_name.clone()).await.unwrap();
    assert!(matches!(rsp, Response::Err(_)));
    assert_eq!(rsp.err(), "No such blob");

    let data = client.download(file.hash, 0, 11).await.unwrap();
    assert!(matches!(data, Response::Ok(_)));
    assert_eq!(data.unwrap(), b"hello world".to_vec());
}
