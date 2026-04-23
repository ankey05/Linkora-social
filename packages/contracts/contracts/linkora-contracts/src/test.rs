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
fn test_tip_zero_fee() {
    let env = Env::default();
    env.mock_all_auths();

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
