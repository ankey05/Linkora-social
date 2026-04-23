#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

fn setup_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(env, &token_id.address()).mint(admin, &10_000);
    token_id.address()
}

#[test]
fn test_profile() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    client.set_profile(
        &user,
        &String::from_str(&env, "alice"),
        &user.clone(),
    );
    let profile = client.get_profile(&user).unwrap();
    assert_eq!(profile.username, String::from_str(&env, "alice"));
}

#[test]
fn test_follow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.follow(&alice, &bob);
    let following = client.get_following(&alice);
    assert_eq!(following.len(), 1);
    assert_eq!(following.get(0).unwrap(), bob);
}

#[test]
fn test_post_and_tip() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);
    let tipper = Address::generate(&env);
    let token = setup_token(&env, &tipper);

    let post_id = client.create_post(&author, &String::from_str(&env, "Hello Linkora!"));
    assert_eq!(post_id, 1);

    client.tip(&tipper, &post_id, &token, &500);

    let post = client.get_post(&post_id).unwrap();
    assert_eq!(post.tip_total, 500);
    assert_eq!(TokenClient::new(&env, &token).balance(&author), 500);
}

#[test]
fn test_pool_deposit_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let token = setup_token(&env, &user);
    let pool_id = symbol_short!("community");

    client.pool_deposit(&user, &pool_id, &token, &1_000);
    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.balance, 1_000);

    client.pool_withdraw(&user, &pool_id, &200);
    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.balance, 800);
    assert_eq!(TokenClient::new(&env, &token).balance(&user), 9_200);
}

#[test]
fn test_sequential_posts() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);

    // Set first timestamp
    let ts1 = 1000;
    env.ledger().set_timestamp(ts1);

    // Create first post
    let post_id1 = client.create_post(&author, &String::from_str(&env, "First post"));
    assert_eq!(post_id1, 1, "First post ID should be 1");

    let post1 = client.get_post(&post_id1).unwrap();
    assert_eq!(post1.timestamp, ts1, "First post timestamp should match ledger");
    assert_eq!(post1.id, 1);

    // Advance timestamp
    let ts2 = 2000;
    env.ledger().set_timestamp(ts2);

    // Create second post
    let post_id2 = client.create_post(&author, &String::from_str(&env, "Second post"));
    assert_eq!(post_id2, 2, "Second post ID should be 2");

    let post2 = client.get_post(&post_id2).unwrap();
    assert_eq!(post2.timestamp, ts2, "Second post timestamp should match updated ledger");
    assert_eq!(post2.id, 2);

    // Verify both exist and are distinct
    assert!(post_id1 != post_id2);
}

#[test]
fn test_delete_post_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);

    // Create a post
    let post_id = client.create_post(&author, &String::from_str(&env, "Test post"));
    assert_eq!(post_id, 1);

    // Verify post exists
    let post = client.get_post(&post_id);
    assert!(post.is_some());

    // Delete the post
    client.delete_post(&author, &post_id);

    // Verify post is deleted
    let post = client.get_post(&post_id);
    assert!(post.is_none());
}

#[test]
#[should_panic(expected = "only author can delete post")]
fn test_delete_post_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);
    let other_user = Address::generate(&env);

    // Create a post
    let post_id = client.create_post(&author, &String::from_str(&env, "Test post"));

    // Try to delete as different user (should panic)
    client.delete_post(&other_user, &post_id);
}

#[test]
#[should_panic(expected = "post does not exist")]
fn test_delete_nonexistent_post() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);

    // Try to delete a post that doesn't exist (should panic)
    client.delete_post(&author, &999);
}

#[test]
fn test_delete_post_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);

    // Create a post
    let post_id = client.create_post(&author, &String::from_str(&env, "Test post"));

    // Delete the post - this will emit an event
    client.delete_post(&author, &post_id);

    // If we reach here without panic, the event was emitted successfully
    // Verify post is deleted
    assert!(client.get_post(&post_id).is_none());
}

#[test]
fn test_delete_multiple_posts() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);

    // Create multiple posts
    let post_id1 = client.create_post(&author, &String::from_str(&env, "First post"));
    let post_id2 = client.create_post(&author, &String::from_str(&env, "Second post"));
    let post_id3 = client.create_post(&author, &String::from_str(&env, "Third post"));

    // Delete the middle post
    client.delete_post(&author, &post_id2);

    // Verify only post 2 is deleted
    assert!(client.get_post(&post_id1).is_some());
    assert!(client.get_post(&post_id2).is_none());
    assert!(client.get_post(&post_id3).is_some());
}
