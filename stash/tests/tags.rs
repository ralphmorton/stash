use std::str::FromStr;

use stash::{Response, Tag};
use util::{ClientServer, TestDb};

mod util;

#[tokio::test]
async fn tag_management() {
    let db = TestDb::new().await;
    let client_server = ClientServer::new(db.pool().clone(), ".".into()).await;
    let client = client_server.client;

    let tags = client.tags().await.unwrap();
    assert!(matches!(tags, Response::Ok(_)));
    assert!(tags.unwrap().is_empty());

    let rsp = client
        .create_tag(Tag::from_str("test-1").unwrap())
        .await
        .unwrap();
    assert!(matches!(rsp, Response::Ok(_)));
    assert_eq!(rsp.unwrap(), "OK");

    let rsp = client
        .create_tag(Tag::from_str("test-2").unwrap())
        .await
        .unwrap();
    assert!(matches!(rsp, Response::Ok(_)));
    assert_eq!(rsp.unwrap(), "OK");

    let tags = client.tags().await.unwrap();
    assert!(matches!(tags, Response::Ok(_)));
    assert_eq!(
        tags.unwrap(),
        vec!["test-1".to_string(), "test-2".to_string()]
    );

    let rsp = client
        .delete_tag(Tag::from_str("test-2").unwrap())
        .await
        .unwrap();
    assert!(matches!(rsp, Response::Ok(_)));
    assert_eq!(rsp.unwrap(), "OK");

    let tags = client.tags().await.unwrap();
    assert!(matches!(tags, Response::Ok(_)));
    assert_eq!(tags.unwrap(), vec!["test-1".to_string()]);
}
