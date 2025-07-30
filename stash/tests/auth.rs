use iroh::{Endpoint, SecretKey, Watcher};
use stash::Response;
use util::{ClientServer, TestInfra};

mod util;

#[tokio::test]
async fn node_auth() {
    let infra = TestInfra::new().await;
    let client_server = ClientServer::new(infra).await;
    let client = client_server.client;

    let rsp = client.all_tags().await.unwrap();
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
        .await
        .unwrap();

    let unauthorized_client = stash::Client::with_addr(client_endpoint, server_addr);

    let rsp = unauthorized_client.all_tags().await;
    assert!(matches!(rsp, Result::Err(_)));
}
