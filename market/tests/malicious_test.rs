use lib_proto::User;
use orcanet_market_ferrous::dht::DhtClient;
use orcanet_market_ferrous::{dht, get_current_time, FileRequest, EXPIRATION_OFFSET};
use rand::{distributions::Alphanumeric, Rng};

fn random_string(len: usize) -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();

    s
}

// start a dht node to bootstrap to before running test
// e.g. `BOOTSTRAP_PEER=/ip4/130.245.173.204/tcp/6881/p2p/{peer_id} cargo test`

async fn create_test_client() -> dht::DhtClient {
    let peer = std::env::var("BOOTSTRAP_PEER").unwrap();
    let bootstrap_peers = vec![peer.parse().unwrap()];

    let listen_on = None;

    let (dht_client, _) = match DhtClient::spawn_client(&bootstrap_peers, listen_on).await {
        Ok(o) => o,
        Err(err) => {
            panic!("Failed to spawn client: {}", err);
        }
    };

    dht_client
}

#[tokio::test]
async fn test_malicious_delete() {
    let dht_client = create_test_client().await;

    //valid request
    let file_hash = random_string(32);
    let user1 = User {
        id: random_string(10),
        name: random_string(10),
        ip: random_string(10),
        port: 1,
        price: 1,
    };
    let valid_exp = get_current_time() + 1000;
    let fr1 = FileRequest {
        user: user1.clone(),
        file_hash: file_hash.clone(),
        expiration: valid_exp,
    };

    let requests = vec![fr1];
    let res = dht_client.set_requests(&file_hash, requests).await;
    assert!(res.is_ok(), "Failed to set valid request");

    // try to "delete" the request by setting expiration to the past
    let fr2 = FileRequest {
        user: user1.clone(),
        file_hash: file_hash.clone(),
        expiration: get_current_time() - 1000,
    };

    let requests = vec![fr2];
    let res = dht_client.set_requests(&file_hash, requests).await;
    assert!(res.is_ok(), "Failed to set request");

    // use a new client to avoid the local map
    drop(dht_client);
    let dht_client2 = create_test_client().await;

    let end_holders = dht_client2.get_requests(&file_hash).await.unwrap();
    match end_holders {
        Some(holders) => {
            assert_eq!(holders.len(), 1);
            assert_eq!(holders[0].expiration, valid_exp);
        }
        None => {
            panic!("Failed to get holders");
        }
    }
}

#[tokio::test]
async fn test_malicious_delete2() {
    let dht_client = create_test_client().await;

    //valid request
    let file_hash = random_string(32);
    let user1 = User {
        id: random_string(10),
        name: random_string(10),
        ip: random_string(10),
        port: 1,
        price: 1,
    };
    let valid_exp = get_current_time() + 1000;
    let fr1 = FileRequest {
        user: user1.clone(),
        file_hash: file_hash.clone(),
        expiration: valid_exp,
    };

    let requests = vec![fr1];
    let _res = dht_client.set_requests(&file_hash, requests).await;

    // try to "delete" the request with an empty vector
    let requests = vec![];
    let _res = dht_client.set_requests(&file_hash, requests).await;

    // use a new client to avoid the local map
    drop(dht_client);
    let dht_client2 = create_test_client().await;

    let end_holders = dht_client2.get_requests(&file_hash).await.unwrap();
    match end_holders {
        Some(holders) => {
            assert_eq!(holders.len(), 1);
            assert_eq!(holders[0].expiration, valid_exp);
        }
        None => {
            panic!("Failed to get holders");
        }
    }
}

#[tokio::test]
async fn test_malicious_duplicate() {
    let dht_client = create_test_client().await;

    //valid request
    let file_hash = random_string(32);
    let user1 = User {
        id: random_string(10),
        name: random_string(10),
        ip: random_string(10),
        port: 1,
        price: 1,
    };
    let valid_exp = get_current_time() + 1000;
    let fr1 = FileRequest {
        user: user1.clone(),
        file_hash: file_hash.clone(),
        expiration: valid_exp,
    };

    let requests = vec![fr1.clone()];
    let _res = dht_client.set_requests(&file_hash, requests).await;

    // try to push a duplicate value
    let requests = vec![fr1.clone(), fr1.clone()];
    let _res = dht_client.set_requests(&file_hash, requests).await;

    // use a new client to avoid the local map
    drop(dht_client);
    let dht_client2 = create_test_client().await;

    let end_holders = dht_client2.get_requests(&file_hash).await.unwrap();
    match end_holders {
        Some(holders) => {
            assert_eq!(holders.len(), 1);
            assert_eq!(holders[0].expiration, valid_exp);
        }
        None => {
            panic!("Failed to get holders");
        }
    }
}

#[tokio::test]
async fn test_mismatched_id() {
    let dht_client = create_test_client().await;

    //valid request
    let file_hash = random_string(32);
    let file_hash_fake = random_string(32);
    let user1 = User {
        id: random_string(10),
        name: random_string(10),
        ip: random_string(10),
        port: 1,
        price: 1,
    };
    let valid_exp = get_current_time() + 1000;
    let fr1 = FileRequest {
        user: user1.clone(),
        file_hash: file_hash.clone(),
        expiration: valid_exp,
    };

    //try to put with a key that doesn't match the file hash
    let requests = vec![fr1.clone()];
    let _res = dht_client.set_requests(&file_hash_fake, requests).await;

    // use a new client to avoid the local map
    drop(dht_client);
    let dht_client2 = create_test_client().await;

    // check the incorrect file hash
    let end_holders = dht_client2.get_requests(&file_hash_fake).await.unwrap();
    match end_holders {
        Some(holders) => {
            assert_eq!(holders.len(), 0);
        }
        None => {}
    }
    // check the file hash that never got assigned to
    let end_holders = dht_client2.get_requests(&file_hash).await.unwrap();
    match end_holders {
        Some(holders) => {
            assert_eq!(holders.len(), 0);
        }
        None => {}
    }
}

#[tokio::test]
async fn test_malicious_merged() {
    // try adding a user, then try to replace it with another user
    // both users should be returned in the next get
    let dht_client = create_test_client().await;

    //valid request
    let file_hash = random_string(32);
    let user1 = User {
        id: random_string(10),
        name: random_string(10),
        ip: random_string(10),
        port: 1,
        price: 1,
    };
    let valid_exp = get_current_time() + 1000;
    let fr1 = FileRequest {
        user: user1.clone(),
        file_hash: file_hash.clone(),
        expiration: valid_exp,
    };

    let requests = vec![fr1.clone()];
    let _res = dht_client.set_requests(&file_hash, requests).await;

    let user2 = User {
        id: random_string(10),
        name: random_string(10),
        ip: random_string(10),
        port: 1,
        price: 1,
    };
    let valid_exp = get_current_time() + 1000;
    let fr2 = FileRequest {
        user: user2.clone(),
        file_hash: file_hash.clone(),
        expiration: valid_exp,
    };
    // try to replace user1
    let requests2 = vec![fr2.clone()];
    let _res = dht_client.set_requests(&file_hash, requests2).await;

    // use a new client to avoid the local map
    drop(dht_client);
    let dht_client2 = create_test_client().await;

    let end_holders = dht_client2.get_requests(&file_hash).await.unwrap();
    match end_holders {
        Some(holders) => {
            assert_eq!(holders.len(), 2);
            assert!(holders.iter().any(|x| *x == fr1));
            assert!(holders.iter().any(|x| *x == fr2));
        }
        None => {
            panic!("Failed to get holders");
        }
    }
}
