#![no_std]
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, 
    Address, Env, String, Symbol, BytesN
};

// Re-export the pure math function so contract callers and tests can use it.
// pub use leaseflow_math::calculate_total_cost; // Only if available in dependencies

macro_rules! require {
    ($condition:expr, $error_msg:expr) => {
        if !$condition {
            panic!($error_msg);
        }
    };
}

// ── Rate helpers ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RateType {
    PerSecond,
    PerHour,
    PerDay,
}

pub fn to_per_second(rate: i128, rate_type: RateType) -> i128 {
    match rate_type {
        RateType::PerSecond => rate,
        RateType::PerHour   => rate / 3_600,
        RateType::PerDay    => rate / 86_400,
    }
}

pub const SECS_PER_UNIT: u64 = 86_400;

// ── Status Enums ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DepositStatus {
    Held,
    Settled,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LeaseStatus {
    Pending,
    Active,
    Expired,
    Disputed,
    Terminated,
}

// ── Structs ───────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseInstance {
    pub landlord: Address,
    pub tenant: Address,
    pub rent_amount: i128,
    pub deposit_amount: i128,
    pub start_date: u64,
    pub end_date: u64,
    pub property_uri: String,
    pub status: LeaseStatus,
    pub nft_contract: Option<Address>,
    pub token_id: Option<u128>,
    pub active: bool,
    pub rent_paid: i128,
    pub debt: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    pub lease_id: Symbol,
    pub month: u32,
    pub amount: i128,
    pub date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseAmendment {
    pub new_rent_amount: Option<i128>,
    pub new_end_date: Option<u64>,
    pub landlord_signature: BytesN<32>,
    pub tenant_signature: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositReleasePartial {
    pub tenant_amount: i128,
    pub landlord_amount: i128,
}

#[contracttype]
pub enum DepositRelease {
    FullRefund,
    PartialRefund(DepositReleasePartial),
    Disputed,
}

// ── Events ────────────────────────────────────────────────────────────────────

#[contractevent]
pub struct LeaseTerminated {
    pub lease_id: Symbol,
}

// ── Storage Keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Lease(Symbol),
    Receipt(Symbol, u32),
    Admin,
}

// ── Storage Helpers ───────────────────────────────────────────────────────────

const DAY_IN_LEDGERS: u32 = 17280; // Assuming 5s ledger time
const MONTH_IN_LEDGERS: u32 = DAY_IN_LEDGERS * 30;
const YEAR_IN_LEDGERS: u32 = DAY_IN_LEDGERS * 365;

pub fn load_lease(env: &Env, lease_id: &Symbol) -> Option<LeaseInstance> {
    env.storage().persistent().get(&DataKey::Lease(lease_id.clone()))
}

pub fn save_lease(env: &Env, lease_id: &Symbol, lease: &LeaseInstance) {
    let key = DataKey::Lease(lease_id.clone());
    env.storage().persistent().set(&key, lease);
    // identities stored in Persistent storage to survive ledger expirations
    env.storage().persistent().extend_ttl(&key, YEAR_IN_LEDGERS, YEAR_IN_LEDGERS);
}

mod nft_contract {
    use soroban_sdk::{contractclient, Address, Env};
    #[contractclient(name = "NftClient")]
    pub trait NftInterface {
        fn transfer_from(env: Env, spender: Address, from: Address, to: Address, token_id: u128);
    }
}

// ── Contract Implementation ───────────────────────────────────────────────────

#[contract]
pub struct LeaseContract;

#[contractimpl]
impl LeaseContract {
    /// Initializes a lease in Persistent storage.
    pub fn initialize_lease(
        env: Env,
        lease_id: Symbol,
        landlord: Address,
        tenant: Address,
        rent_amount: i128,
        deposit_amount: i128,
        duration: u64,
        property_uri: String,
    ) -> bool {
        landlord.require_auth();

        let start_date = env.ledger().timestamp();
        let end_date = start_date.saturating_add(duration);

        let lease = LeaseInstance {
            landlord,
            tenant,
            rent_amount,
            deposit_amount,
            start_date,
            end_date,
            property_uri,
            status: LeaseStatus::Pending,
            nft_contract: None,
            token_id: None,
            active: true,
            rent_paid: 0,
            debt: 0,
        };

        save_lease(&env, &lease_id, &lease);
        true
    }

    /// Processes rent payment, saves receipt in Instance storage, and extends TTL.
    pub fn pay_rent(env: Env, lease_id: Symbol, month: u32, amount: i128) -> bool {
        let mut lease = load_lease(&env, &lease_id).expect("Lease not found");
        lease.tenant.require_auth();

        // Monthly payment receipts use Instance storage to keep costs low
        let receipt = Receipt {
            lease_id: lease_id.clone(),
            month,
            amount,
            date: env.ledger().timestamp(),
        };
        
        env.storage().instance().set(&DataKey::Receipt(lease_id.clone(), month), &receipt);

        lease.rent_paid += amount;
        save_lease(&env, &lease_id, &lease);

        // Keep the contract "alive" for the duration of the lease
        env.storage().instance().extend_ttl(MONTH_IN_LEDGERS, YEAR_IN_LEDGERS);
        
        true
    }

    pub fn get_lease(env: Env, lease_id: Symbol) -> LeaseInstance {
        load_lease(&env, &lease_id).expect("Lease not found")
    }

    pub fn get_receipt(env: Env, lease_id: Symbol, month: u32) -> Receipt {
        env.storage()
            .instance()
            .get(&DataKey::Receipt(lease_id, month))
            .expect("Receipt not found")
    }

    pub fn activate_lease(env: Env, lease_id: Symbol, tenant: Address) -> bool {
        let mut lease = load_lease(&env, &lease_id).expect("Lease not found");
        require!(lease.tenant == tenant, "Unauthorized");
        lease.status = LeaseStatus::Active;
        save_lease(&env, &lease_id, &lease);
        true
    }

    pub fn extend_ttl(env: Env, lease_id: Symbol) {
        let key = DataKey::Lease(lease_id);
        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(&key, MONTH_IN_LEDGERS, YEAR_IN_LEDGERS);
        }
        env.storage().instance().extend_ttl(MONTH_IN_LEDGERS, YEAR_IN_LEDGERS);
    }
}

mod test;
