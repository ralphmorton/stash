use iroh::{Endpoint, SecretKey, Watcher};
use stash::Response;
use util::{ClientServer, TestInfra};

mod util;

#[tokio::test]
async fn node_auth() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra).await;
    let client = client_server.client;

    let rsp = client.tags().await.unwrap();
    assert!(matches!(rsp, Response::Ok(_)));

    let mut rng = rand::thread_rng();
    let client_sk = SecretKey::generate(&mut rng);
    let client_endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(client_sk.clone())
        .bind()
        .await
        .unwrap();

    let server_addr = client_server
        .server
        .endpoint()
        .node_addr()
        .initialized()
        .await;

    let other_client = stash::Client::with_addr(client_endpoint, server_addr);

    let rsp = other_client.tags().await;
    assert!(matches!(rsp, Result::Err(_)));

    let rsp = client.add_client(client_sk.public()).await.unwrap();
    assert!(matches!(rsp, Response::Ok(_)));

    let rsp = other_client.tags().await.unwrap();
    assert!(matches!(rsp, Response::Ok(_)));

    let rsp = other_client.add_client(client_sk.public()).await.unwrap();
    assert!(matches!(rsp, Response::Err(_)));
    assert_eq!(rsp.err(), "Unauthorized");

    let rsp = other_client
        .remove_client(client_sk.public())
        .await
        .unwrap();
    assert!(matches!(rsp, Response::Err(_)));
    assert_eq!(rsp.err(), "Unauthorized");

    let rsp = client.remove_client(client_sk.public()).await.unwrap();
    assert!(matches!(rsp, Response::Ok(_)));

    let rsp = other_client.tags().await;
    assert!(matches!(rsp, Result::Err(_)));
}
