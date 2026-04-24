#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env, String,
};

fn setup_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(env, &token_id.address()).mint(admin, &10_000);
    token_id.address()
}

fn setup_contract(env: &Env) -> (LinkoraContractClient, Address, Address) {
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.initialize(&admin, &treasury, &0);
    (client, admin, treasury)
}

#[test]
fn test_tip_fee_split() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let author = Address::generate(&env);
    let tipper = Address::generate(&env);

    // Initialize with 2.5% fee (250 bps)
    client.initialize(&admin, &treasury, &250);

    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "Fee test post"));

    // Tip 1000 units
    client.tip(&tipper, &post_id, &token, &1000);

    // Verify balances
    // Fee = 1000 * 250 / 10000 = 25
    // Author gets 1000 - 25 = 975
    assert_eq!(TokenClient::new(&env, &token).balance(&treasury), 25);
    assert_eq!(TokenClient::new(&env, &token).balance(&author), 975);

    let post = client.get_post(&post_id).unwrap();
    assert_eq!(post.tip_total, 1000);
}

#[test]
fn test_profile_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = Address::generate(&env);

    client.set_profile(&user1, &String::from_str(&env, "alice"), &token);
    assert_eq!(client.get_profile_count(), 1);

    // Update profile should not increment count
    client.set_profile(&user1, &String::from_str(&env, "alice_new"), &token);
    assert_eq!(client.get_profile_count(), 1);

    client.set_profile(&user2, &String::from_str(&env, "bob"), &token);
    assert_eq!(client.get_profile_count(), 2);
}

#[test]
fn test_post_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let author = Address::generate(&env);
    client.create_post(&author, &String::from_str(&env, "Post 1"));
    client.create_post(&author, &String::from_str(&env, "Post 2"));

    assert_eq!(client.get_post_count(), 2);
}

#[test]
fn test_pool_authorization() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let pool_admin1 = Address::generate(&env);
    let pool_admin2 = Address::generate(&env);
    let other_user = Address::generate(&env);
    // Mint tokens to the admin who will distribute or just use the pool_admin for deposit
    let token = setup_token(&env, &pool_admin1);

    // Give other_user some tokens to deposit
    StellarAssetClient::new(&env, &token).mint(&other_user, &1000);

    let pool_id = symbol_short!("pool1");
    // Create pool with 2-of-2 threshold
    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, pool_admin1.clone(), pool_admin2.clone()],
        &2,
    );

    // Deposit works for anyone with tokens
    client.pool_deposit(&other_user, &pool_id, &token, &100);

    // Withdrawal by both admins works
    client.pool_withdraw(
        &vec![&env, pool_admin1.clone(), pool_admin2.clone()],
        &pool_id,
        &50,
        &other_user,
    );
    assert_eq!(client.get_pool(&pool_id).unwrap().balance, 50);
}

#[test]
#[should_panic(expected = "insufficient signers")]
fn test_pool_withdraw_insufficient_signers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let pool_admin1 = Address::generate(&env);
    let pool_admin2 = Address::generate(&env);
    let other_user = Address::generate(&env);
    let token = setup_token(&env, &pool_admin1);
    StellarAssetClient::new(&env, &token).mint(&other_user, &1000);

    let pool_id = symbol_short!("pool1");
    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, pool_admin1.clone(), pool_admin2.clone()],
        &2,
    );
    client.pool_deposit(&other_user, &pool_id, &token, &100);

    // Only 1 signer when 2 required
    client.pool_withdraw(&vec![&env, pool_admin1.clone()], &pool_id, &50, &other_user);
}

#[test]
fn test_pool_admin_update() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup_contract(&env);

    let pool_admin1 = Address::generate(&env);
    let pool_admin2 = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let token = setup_token(&env, &pool_admin1);

    let pool_id = symbol_short!("pool1");
    client.create_pool(
        &admin,
        &pool_id,
        &token,
        &vec![&env, pool_admin1.clone(), pool_admin2.clone()],
        &2,
    );

    // Update admins to just new_admin with threshold 1
    client.update_pool_admins(
        &vec![&env, pool_admin1.clone(), pool_admin2.clone()],
        &pool_id,
        &vec![&env, new_admin.clone()],
        &1,
    );

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.admins.len(), 1);
    assert_eq!(pool.admins.get(0).unwrap(), new_admin);
    assert_eq!(pool.threshold, 1);
}

#[test]
fn test_event_emission() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let author = Address::generate(&env);
    client.create_post(&author, &String::from_str(&env, "Event test post"));

    let events = env.events().all();
    // In v25, ContractEvents can be compared with a Vec.
    // If it's not empty, it should not be equal to an empty Vec.
    assert_ne!(events, vec![&env]);
}

#[test]
fn test_sequential_posts() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let author = Address::generate(&env);

    // Set first timestamp
    let ts1 = 1000;
    env.ledger().set_timestamp(ts1);

    // Create first post
    let post_id1 = client.create_post(&author, &String::from_str(&env, "First post"));
    assert_eq!(post_id1, 1, "First post ID should be 1");

    let post1 = client.get_post(&post_id1).unwrap();
    assert_eq!(
        post1.timestamp, ts1,
        "First post timestamp should match ledger"
    );
    assert_eq!(post1.id, 1);

    // Advance timestamp
    let ts2 = 2000;
    env.ledger().set_timestamp(ts2);

    // Create second post
    let post_id2 = client.create_post(&author, &String::from_str(&env, "Second post"));
    assert_eq!(post_id2, 2, "Second post ID should be 2");

    let post2 = client.get_post(&post_id2).unwrap();
    assert_eq!(
        post2.timestamp, ts2,
        "Second post timestamp should match updated ledger"
    );
    assert_eq!(post2.id, 2);

    // Verify both exist and are distinct
    assert!(post_id1 != post_id2);
}

#[test]
fn test_follow_is_idempotent() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup_contract(&env);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Follow bob twice from alice — should be deduplicated
    client.follow(&alice, &bob);
    client.follow(&alice, &bob);

    let following = client.get_following(&alice);
    // Bob must appear exactly once despite two follow calls
    assert_eq!(following.len(), 1);
    assert_eq!(following.get(0).unwrap(), bob);
}
