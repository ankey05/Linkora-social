#![no_std]
use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env,
    String, Symbol, Vec,
};

// ── Storage Keys ────────────────────────────────────────────────────────────

const POSTS: Symbol = symbol_short!("POSTS");
const POST_CT: Symbol = symbol_short!("POST_CT");
const PROFILES: Symbol = symbol_short!("PROFILES");
const PROFILE_CT: Symbol = symbol_short!("PROF_CT");
const FOLLOWS: Symbol = symbol_short!("FOLLOWS");
const FOLLOWERS: Symbol = symbol_short!("FOLLOWRS");
const POOLS: Symbol = symbol_short!("POOLS");
const ADMIN: Symbol = symbol_short!("ADMIN");
const TREASURY: Symbol = symbol_short!("TREASURY");
const FEE_BPS: Symbol = symbol_short!("FEE_BPS");
const INITIALIZED: Symbol = symbol_short!("INIT");

// ── Validation Constants ─────────────────────────────────────────────────────

const MIN_USERNAME_LEN: u32 = 3;
const MAX_USERNAME_LEN: u32 = 32;
const MIN_CONTENT_LEN: u32 = 1;
const MAX_CONTENT_LEN: u32 = 280;

// ── Data Types ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct Post {
    pub id: u64,
    pub author: Address,
    pub content: String,
    pub tip_total: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Profile {
    pub address: Address,
    pub username: String,
    pub creator_token: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Pool {
    pub token: Address,
    pub balance: i128,
    pub admins: Vec<Address>,
    pub threshold: u32,
}

// ── Events ───────────────────────────────────────────────────────────────────

#[contractevent]
pub struct ProfileSetEvent {
    #[topic]
    pub user: Address,
    pub username: String,
}

#[contractevent]
pub struct FollowEvent {
    #[topic]
    pub follower: Address,
    #[topic]
    pub followee: Address,
}

#[contractevent]
pub struct UnfollowEvent {
    #[topic]
    pub follower: Address,
    #[topic]
    pub followee: Address,
}

#[contractevent]
pub struct PostCreatedEvent {
    #[topic]
    pub id: u64,
    #[topic]
    pub author: Address,
}

#[contractevent]
pub struct TipEvent {
    #[topic]
    pub tipper: Address,
    #[topic]
    pub post_id: u64,
    pub amount: i128,
    pub fee: i128,
}

#[contractevent]
pub struct PoolDepositEvent {
    #[topic]
    pub depositor: Address,
    #[topic]
    pub pool_id: Symbol,
    pub amount: i128,
}

#[contractevent]
pub struct PoolWithdrawEvent {
    #[topic]
    pub recipient: Address,
    #[topic]
    pub pool_id: Symbol,
    pub amount: i128,
}

#[contractevent]
pub struct ContractUpgraded {
    pub new_wasm_hash: BytesN<32>,
}

#[contractevent]
pub struct PostDeleted {
    #[topic]
    pub post_id: u64,
    #[topic]
    pub author: Address,
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct LinkoraContract;

// ── Validation Helpers ───────────────────────────────────────────────────────

fn validate_username(username: &String) -> Result<(), &'static str> {
    let len = username.len();
    if len < MIN_USERNAME_LEN {
        return Err("username too short");
    }
    if len > MAX_USERNAME_LEN {
        return Err("username too long");
    }
    let bytes = username.to_bytes();
    for i in 0..bytes.len() {
        let byte = bytes.get(i).unwrap();
        let c = byte as char;
        if !c.is_ascii_alphanumeric() && c != '_' {
            return Err("invalid characters");
        }
    }
    Ok(())
}

fn validate_content(content: &String) -> Result<(), &'static str> {
    let len = content.len();
    if len < MIN_CONTENT_LEN {
        return Err("empty content");
    }
    if len > MAX_CONTENT_LEN {
        return Err("content too long");
    }
    Ok(())
}

#[contractimpl]
impl LinkoraContract {
    // ── Profiles ─────────────────────────────────────────────────────────────

    pub fn set_profile(env: Env, user: Address, username: String, creator_token: Address) {
        user.require_auth();
        validate_username(&username).expect("invalid username");

        let key = (PROFILES, user.clone());
        if !env.storage().persistent().has(&key) {
            let count: u64 = env.storage().instance().get(&PROFILE_CT).unwrap_or(0);
            env.storage().instance().set(&PROFILE_CT, &(count + 1));
        }

        env.storage().persistent().set(
            &key,
            &Profile {
                address: user.clone(),
                username: username.clone(),
                creator_token,
            },
        );

        ProfileSetEvent { user, username }.publish(&env);
    }

    pub fn get_profile(env: Env, user: Address) -> Option<Profile> {
        env.storage().persistent().get(&(PROFILES, user))
    }

    pub fn get_profile_count(env: Env) -> u64 {
        env.storage().instance().get(&PROFILE_CT).unwrap_or(0)
    }

    // ── Social Graph ─────────────────────────────────────────────────────────

    pub fn follow(env: Env, follower: Address, followee: Address) {
        follower.require_auth();
        let following_key = (FOLLOWS, follower.clone());
        let mut following_list: Vec<Address> = env
            .storage()
            .persistent()
            .get(&following_key)
            .unwrap_or(Vec::new(&env));

        if !following_list.contains(&followee) {
            following_list.push_back(followee.clone());
            env.storage()
                .persistent()
                .set(&following_key, &following_list);

            let followers_key = (FOLLOWERS, followee.clone());
            let mut followers_list: Vec<Address> = env
                .storage()
                .persistent()
                .get(&followers_key)
                .unwrap_or(Vec::new(&env));
            followers_list.push_back(follower.clone());
            env.storage()
                .persistent()
                .set(&followers_key, &followers_list);
        }

        FollowEvent { follower, followee }.publish(&env);
    }

    pub fn unfollow(env: Env, follower: Address, followee: Address) {
        follower.require_auth();
        let following_key = (FOLLOWS, follower.clone());
        let mut following_list: Vec<Address> = env
            .storage()
            .persistent()
            .get(&following_key)
            .unwrap_or(Vec::new(&env));

        if let Some(index) = following_list.iter().position(|addr| addr == followee) {
            following_list.remove(index as u32);
            env.storage()
                .persistent()
                .set(&following_key, &following_list);

            let followers_key = (FOLLOWERS, followee.clone());
            let mut followers_list: Vec<Address> = env
                .storage()
                .persistent()
                .get(&followers_key)
                .unwrap_or(Vec::new(&env));
            if let Some(f_index) = followers_list.iter().position(|addr| addr == follower) {
                followers_list.remove(f_index as u32);
                env.storage()
                    .persistent()
                    .set(&followers_key, &followers_list);
            }
        }

        UnfollowEvent { follower, followee }.publish(&env);
    }

    pub fn get_following(env: Env, user: Address) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&(FOLLOWS, user))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_followers(env: Env, user: Address) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&(FOLLOWERS, user))
            .unwrap_or(Vec::new(&env))
    }

    // ── Posts ─────────────────────────────────────────────────────────────────

    pub fn create_post(env: Env, author: Address, content: String) -> u64 {
        author.require_auth();
        validate_content(&content).expect("invalid content");

        let id: u64 = env.storage().instance().get(&POST_CT).unwrap_or(0) + 1;
        let post = Post {
            id,
            author: author.clone(),
            content,
            tip_total: 0,
            timestamp: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&(POSTS, id), &post);
        env.storage().instance().set(&POST_CT, &id);

        PostCreatedEvent { id, author }.publish(&env);
        id
    }

    pub fn get_post(env: Env, id: u64) -> Option<Post> {
        env.storage().persistent().get(&(POSTS, id))
    }

    pub fn get_post_count(env: Env) -> u64 {
        env.storage().instance().get(&POST_CT).unwrap_or(0)
    }

    pub fn delete_post(env: Env, author: Address, post_id: u64) {
        author.require_auth();
        let key = (POSTS, post_id);
        let post: Post = env
            .storage()
            .persistent()
            .get(&key)
            .expect("post not found");
        assert!(post.author == author, "not author");
        env.storage().persistent().remove(&key);
        PostDeleted { post_id, author }.publish(&env);
    }

    // ── Tipping ───────────────────────────────────────────────────────────────

    pub fn tip(env: Env, tipper: Address, post_id: u64, token: Address, amount: i128) {
        tipper.require_auth();
        let key = (POSTS, post_id);
        let mut post: Post = env
            .storage()
            .persistent()
            .get(&key)
            .expect("post not found");

        let fee_bps: u32 = env.storage().instance().get(&FEE_BPS).unwrap_or(0);
        let treasury: Option<Address> = env.storage().instance().get(&TREASURY);

        let fee_amount = if let Some(ref _t) = treasury {
            (amount * (fee_bps as i128)) / 10_000
        } else {
            0
        };
        let author_amount = amount - fee_amount;
        let token_client = token::Client::new(&env, &token);

        if fee_amount > 0 {
            if let Some(treasury_addr) = treasury {
                token_client.transfer(&tipper, &treasury_addr, &fee_amount);
            }
        }
        token_client.transfer(&tipper, &post.author, &author_amount);

        post.tip_total += amount;
        env.storage().persistent().set(&key, &post);

        TipEvent {
            tipper,
            post_id,
            amount,
            fee: fee_amount,
        }
        .publish(&env);
    }

    // ── Community Token Pool ──────────────────────────────────────────────────

    pub fn create_pool(
        env: Env,
        admin: Address,
        pool_id: Symbol,
        token: Address,
        initial_admins: Vec<Address>,
        threshold: u32,
    ) {
        admin.require_auth(); // Ensure the designated admin authorizes
        Self::require_admin(&env); // Ensure contract admin authorizes
        let key = (POOLS, pool_id);
        assert!(!env.storage().persistent().has(&key), "pool exists");
        assert!(
            threshold > 0 && threshold <= initial_admins.len(),
            "invalid threshold"
        );

        env.storage().persistent().set(
            &key,
            &Pool {
                token,
                balance: 0,
                admins: initial_admins,
                threshold,
            },
        );
    }

    pub fn pool_deposit(
        env: Env,
        depositor: Address,
        pool_id: Symbol,
        token: Address,
        amount: i128,
    ) {
        assert!(amount > 0, "must be positive");
        depositor.require_auth();
        let key = (POOLS, pool_id.clone());
        let mut pool: Pool = env
            .storage()
            .persistent()
            .get(&key)
            .expect("pool not found");
        assert!(pool.token == token, "wrong token");

        token::Client::new(&env, &token).transfer(
            &depositor,
            &env.current_contract_address(),
            &amount,
        );
        pool.balance += amount;
        env.storage().persistent().set(&key, &pool);

        PoolDepositEvent {
            depositor,
            pool_id,
            amount,
        }
        .publish(&env);
    }

    pub fn pool_withdraw(
        env: Env,
        signers: Vec<Address>,
        pool_id: Symbol,
        amount: i128,
        recipient: Address,
    ) {
        assert!(amount > 0, "must be positive");
        let key = (POOLS, pool_id.clone());
        let mut pool: Pool = env
            .storage()
            .persistent()
            .get(&key)
            .expect("pool not found");

        assert!(signers.len() >= pool.threshold, "insufficient signers");
        for signer in signers.iter() {
            assert!(pool.admins.contains(&signer), "unauthorized signer");
            signer.require_auth();
        }
        assert!(pool.balance >= amount, "low balance");

        pool.balance -= amount;
        env.storage().persistent().set(&key, &pool);
        token::Client::new(&env, &pool.token).transfer(
            &env.current_contract_address(),
            &recipient,
            &amount,
        );

        PoolWithdrawEvent {
            recipient,
            pool_id,
            amount,
        }
        .publish(&env);
    }

    pub fn update_pool_admins(
        env: Env,
        signers: Vec<Address>,
        pool_id: Symbol,
        new_admins: Vec<Address>,
        new_threshold: u32,
    ) {
        let key = (POOLS, pool_id);
        let mut pool: Pool = env
            .storage()
            .persistent()
            .get(&key)
            .expect("pool not found");

        assert!(signers.len() >= pool.threshold, "insufficient signers");
        for signer in signers.iter() {
            assert!(pool.admins.contains(&signer), "unauthorized signer");
            signer.require_auth();
        }

        assert!(
            new_threshold > 0 && new_threshold <= new_admins.len(),
            "invalid threshold"
        );

        pool.admins = new_admins;
        pool.threshold = new_threshold;
        env.storage().persistent().set(&key, &pool);
    }

    pub fn get_pool(env: Env, pool_id: Symbol) -> Option<Pool> {
        env.storage().persistent().get(&(POOLS, pool_id))
    }

    // ── Upgradability ─────────────────────────────────────────────────────────

    pub fn initialize(env: Env, admin: Address, treasury: Address, fee_bps: u32) {
        if env
            .storage()
            .instance()
            .get::<Symbol, bool>(&INITIALIZED)
            .unwrap_or(false)
        {
            panic!("already initialized");
        }
        assert!(fee_bps <= 10000, "invalid fee");
        env.storage().instance().set(&INITIALIZED, &true);
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&TREASURY, &treasury);
        env.storage().instance().set(&FEE_BPS, &fee_bps);
    }

    pub fn set_fee(env: Env, fee_bps: u32) {
        Self::require_admin(&env);
        assert!(fee_bps <= 10000, "invalid fee");
        env.storage().instance().set(&FEE_BPS, &fee_bps);
    }

    pub fn set_treasury(env: Env, treasury: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&TREASURY, &treasury);
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        Self::require_admin(&env);
        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        ContractUpgraded { new_wasm_hash }.publish(&env);
    }

    fn require_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .expect("not initialized");
        admin.require_auth();
    }
}

mod test;
