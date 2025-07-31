use std::str::FromStr;

use stash::{Response, Tag};
use util::{ClientServer, TestInfra};

mod util;

#[tokio::test]
async fn tags() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra).await;
    let client = client_server.client;

    let rsp = client.tags().await.unwrap();
    assert!(matches!(rsp, Response::Ok(_)));
    assert!(rsp.unwrap().is_empty());

    let blob = client.create_blob().await.unwrap().unwrap();
    let blob = client
        .append_blob(blob.name, b"hello".to_vec())
        .await
        .unwrap()
        .unwrap();

    client
        .commit_blob(
            blob.name,
            "f".to_string(),
            vec![Tag::from_str("t1").unwrap()],
        )
        .await
        .unwrap()
        .unwrap();

    let rsp = client.tags().await.unwrap();
    assert!(matches!(rsp, Response::Ok(_)));
    assert_eq!(rsp.unwrap(), vec!["t1".to_string()]);
}
