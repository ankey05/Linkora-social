#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{storage::Persistent as _, Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

fn setup_env() -> Env {
    let env = Env::default();
    env.ledger().with_mut(|li| {
        li.sequence_number = 100_000;
        li.min_persistent_entry_ttl = 500;
        li.max_entry_ttl = 600_000;
    });
    env
}

fn setup_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(env, &token_id.address()).mint(admin, &10_000);
    token_id.address()
}

// ── Profile tests ─────────────────────────────────────────────────────────────

#[test]
fn test_set_and_get_profile() {
    let env = Env::default();
    env.mock_all_auths();
<<<<<<< HEAD
=======

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let author = Address::generate(&env);
    let tipper = Address::generate(&env);
    
    // Initialize with 0% fee
    client.initialize(&admin, &treasury, &0);

    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "Zero fee post"));

    client.tip(&tipper, &post_id, &token, &1000);

    assert_eq!(TokenClient::new(&env, &token).balance(&treasury), 0);
    assert_eq!(TokenClient::new(&env, &token).balance(&author), 1000);
}

#[test]
fn test_set_fee_and_treasury() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    
    client.initialize(&admin, &treasury, &0);

    // Update fee
    client.set_fee(&500); // 5%
    
    // Update treasury
    let new_treasury = Address::generate(&env);
    client.set_treasury(&new_treasury);

    let author = Address::generate(&env);
    let tipper = Address::generate(&env);
    let token = setup_token(&env, &tipper);
    let post_id = client.create_post(&author, &String::from_str(&env, "Update test post"));

    client.tip(&tipper, &post_id, &token, &1000);

    assert_eq!(TokenClient::new(&env, &token).balance(&new_treasury), 50);
    assert_eq!(TokenClient::new(&env, &token).balance(&author), 950);
}

#[test]
#[should_panic(expected = "fee_bps cannot exceed 10000")]
fn test_invalid_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.initialize(&admin, &treasury, &10001);
}

#[test]
#[should_panic(expected = "deposit amount must be positive")]
fn test_pool_deposit_zero_amount() {
=======
fn test_sequential_posts() {
>>>>>>> main
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

<<<<<<< fix/reject-zero-negative-pool-withdrawal
    let user = Address::generate(&env);
    let token = setup_token(&env, &user);
    let pool_id = symbol_short!("community");

    // Zero deposit must be rejected before any state change
    client.pool_deposit(&user, &pool_id, &token, &0);
}

#[test]
#[should_panic(expected = "deposit amount must be positive")]
fn test_pool_deposit_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();

>>>>>>> 7e386b3 (Add block_user function to restrict unwanted follow and tip interactions)
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_profile(&user, &String::from_str(&env, "alice"), &token);

    let profile = client.get_profile(&user).unwrap();
    assert_eq!(profile.username, String::from_str(&env, "alice"));
}

// ── Follow tests ──────────────────────────────────────────────────────────────

#[test]
fn test_follow_is_idempotent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.follow(&alice, &bob);
    client.follow(&alice, &bob);

    let following = client.get_following(&alice);
    assert_eq!(following.len(), 1);
    assert_eq!(following.get(0).unwrap(), bob);
}

<<<<<<< HEAD
// ── Post tests ────────────────────────────────────────────────────────────────

#[test]
fn test_sequential_posts() {
=======
#[test]
fn test_block_user() {
>>>>>>> 7e386b3 (Add block_user function to restrict unwanted follow and tip interactions)
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);
<<<<<<< HEAD
    let author = Address::generate(&env);

    env.ledger().set_timestamp(1000);
    let id1 = client.create_post(&author, &String::from_str(&env, "First post"));
    assert_eq!(id1, 1);
    assert_eq!(client.get_post(&id1).unwrap().timestamp, 1000);

    env.ledger().set_timestamp(2000);
    let id2 = client.create_post(&author, &String::from_str(&env, "Second post"));
    assert_eq!(id2, 2);
    assert_eq!(client.get_post(&id2).unwrap().timestamp, 2000);
}
#[test]
fn test_get_post_count_after_create_and_delete() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);
    let author = Address::generate(&env);

    assert_eq!(client.get_post_count(), 0);

    let id1 = client.create_post(&author, &String::from_str(&env, "First post"));
    assert_eq!(client.get_post_count(), 1);

    let _id2 = client.create_post(&author, &String::from_str(&env, "Second post"));
    assert_eq!(client.get_post_count(), 2);

    client.delete_post(&author, &id1);
    assert_eq!(client.get_post_count(), 2);
}
// ── Pool tests ────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "deposit amount must be positive")]
fn test_pool_deposit_zero_amount() {
=======

    let blocker = Address::generate(&env);
    let blocked = Address::generate(&env);

    // Block
    client.block_user(&blocker, &blocked);
    assert!(client.is_blocked(&blocker, &blocked));

    // Unblock
    client.unblock_user(&blocker, &blocked);
    assert!(!client.is_blocked(&blocker, &blocked));
}

#[test]
#[should_panic(expected = "blocked")]
fn test_follow_after_block() {
>>>>>>> 7e386b3 (Add block_user function to restrict unwanted follow and tip interactions)
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);
<<<<<<< HEAD
    let user = Address::generate(&env);
    let token = setup_token(&env, &user);
    client.pool_deposit(&user, &symbol_short!("community"), &token, &0);
}

#[test]
#[should_panic(expected = "deposit amount must be positive")]
fn test_pool_deposit_negative_amount() {
=======

    let blocker = Address::generate(&env);
    let blocked = Address::generate(&env);

    // Blocker blocks blocked
    client.block_user(&blocker, &blocked);

    // Blocked tries to follow blocker — should panic
    client.follow(&blocked, &blocker);
}

#[test]
fn test_follow_after_unblock() {
>>>>>>> 7e386b3 (Add block_user function to restrict unwanted follow and tip interactions)
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);
<<<<<<< HEAD
    let user = Address::generate(&env);
    let token = setup_token(&env, &user);
    client.pool_deposit(&user, &symbol_short!("community"), &token, &-1);
}

#[test]
#[should_panic(expected = "withdrawal amount must be positive")]
fn test_pool_withdraw_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let token = setup_token(&env, &user);
    client.pool_deposit(&user, &symbol_short!("community"), &token, &1_000);
    client.pool_withdraw(&user, &symbol_short!("community"), &0);
}

#[test]
#[should_panic(expected = "withdrawal amount must be positive")]
fn test_pool_withdraw_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let token = setup_token(&env, &user);
    client.pool_deposit(&user, &symbol_short!("community"), &token, &1_000);
    client.pool_withdraw(&user, &symbol_short!("community"), &-1);
}

// ── TTL tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_ttl_extended_after_set_profile() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_profile(&user, &String::from_str(&env, "alice"), &token);

    env.as_contract(&contract_id, || {
        let key = (PROFILES, user.clone());
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= LEDGER_THRESHOLD, "profile TTL should be bumped after set");
    });
}

#[test]
fn test_ttl_extended_on_get_profile() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_profile(&user, &String::from_str(&env, "alice"), &token);

    // Advance ledger so TTL has partially decayed
    env.ledger().with_mut(|li| li.sequence_number += 100_000);

    client.get_profile(&user);

    env.as_contract(&contract_id, || {
        let key = (PROFILES, user.clone());
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= LEDGER_THRESHOLD, "profile TTL should be bumped on get");
    });
}

#[test]
fn test_ttl_extended_after_create_post() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);
    let post_id = client.create_post(&author, &String::from_str(&env, "hello"));

    env.as_contract(&contract_id, || {
        let key = (POSTS, post_id);
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= LEDGER_THRESHOLD, "post TTL should be bumped after create");
    });
}

#[test]
fn test_ttl_extended_on_get_post() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);
    let post_id = client.create_post(&author, &String::from_str(&env, "hello"));

    env.ledger().with_mut(|li| li.sequence_number += 100_000);

    client.get_post(&post_id);

    env.as_contract(&contract_id, || {
        let key = (POSTS, post_id);
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= LEDGER_THRESHOLD, "post TTL should be bumped on get");
    });
}

#[test]
fn test_ttl_extended_after_pool_deposit() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let token = setup_token(&env, &user);
    let pool_id = symbol_short!("pool1");
    client.pool_deposit(&user, &pool_id, &token, &500);

    env.as_contract(&contract_id, || {
        let key = (POOLS, pool_id.clone());
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= LEDGER_THRESHOLD, "pool TTL should be bumped after deposit");
    });
}

#[test]
fn test_ttl_extended_on_get_pool() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let token = setup_token(&env, &user);
    let pool_id = symbol_short!("pool1");
    client.pool_deposit(&user, &pool_id, &token, &500);

    env.ledger().with_mut(|li| li.sequence_number += 100_000);

    client.get_pool(&pool_id);

    env.as_contract(&contract_id, || {
        let key = (POOLS, pool_id.clone());
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= LEDGER_THRESHOLD, "pool TTL should be bumped on get");
    });
}

#[test]
fn test_ttl_extended_after_follow() {
    let env = setup_env();
    env.mock_all_auths();
    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.follow(&alice, &bob);

    env.as_contract(&contract_id, || {
        let key = (FOLLOWS, alice.clone());
        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= LEDGER_THRESHOLD, "follow list TTL should be bumped after follow");
    });
}

#[test]
fn test_like_post() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(LinkoraContract, ());
    let client = LinkoraContractClient::new(&env, &contract_id);

    let author = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let post_id = client.create_post(&author, &String::from_str(&env, "Test post"));

    // First like
    client.like_post(&user1, &post_id);
    assert_eq!(client.get_like_count(&post_id), 1);
    assert!(client.has_liked(&user1, &post_id));

    // Duplicate like (should be idempotent)
    client.like_post(&user1, &post_id);
    assert_eq!(client.get_like_count(&post_id), 1);

    // Another user likes
    client.like_post(&user2, &post_id);
    assert_eq!(client.get_like_count(&post_id), 2);
    assert!(client.has_liked(&user2, &post_id));
=======

    let blocker = Address::generate(&env);
    let blocked = Address::generate(&env);

    // Block
    client.block_user(&blocker, &blocked);

    // Unblock
    client.unblock_user(&blocker, &blocked);

    // Now follow should work
    client.follow(&blocked, &blocker);

    let followers = client.get_followers(&blocker);
    assert_eq!(followers.len(), 1);
    assert_eq!(followers.get(0).unwrap(), blocked);
>>>>>>> 7e386b3 (Add block_user function to restrict unwanted follow and tip interactions)
}
