#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, BytesN, String};

macro_rules! require {
    ($condition:expr, $error_msg:expr) => {
        if !$condition {
            panic!($error_msg);
        }
    };
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LeaseStatus {
    Pending,
    Active,
    Expired,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lease {
    pub landlord: Address,
    pub tenant: Address,
    pub rent_amount: i128,
    pub deposit_amount: i128,
    pub start_date: u64,
    pub end_date: u64,
    pub property_uri: String,
    pub status: LeaseStatus,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DepositRelease {
    FullRefund,
    PartialRefund(DepositReleasePartial),
    Disputed,
}

#[contract]
pub struct LeaseContract;

#[contractimpl]
impl LeaseContract {
    /// Initializes a lease with collateral lock (security deposit)
    pub fn initialize_lease(
        env: Env,
        landlord: Address,
        tenant: Address,
        rent_amount: i128,
        deposit_amount: i128,
        start_date: u64,
        end_date: u64,
        property_uri: String,
    ) -> Symbol {
        let lease = Lease {
            landlord: landlord.clone(),
            tenant: tenant.clone(),
            rent_amount,
            deposit_amount,
            start_date,
            end_date,
            property_uri: property_uri.clone(),
            status: LeaseStatus::Pending,
        };
        
        env.storage()
            .instance()
            .set(&symbol_short!("lease"), &lease);
        
        symbol_short!("pending")
    }
    
    /// Activates lease after security deposit is transferred
    pub fn activate_lease(env: Env, tenant: Address) -> Symbol {
        let mut lease = Self::get_lease(env.clone());
        
        require!(lease.tenant == tenant, "Unauthorized: Only tenant can activate lease");
        require!(lease.status == LeaseStatus::Pending, "Lease is not in pending state");
        
        // In a real implementation, this would verify the token transfer
        // For now, we'll assume the deposit has been transferred
        lease.status = LeaseStatus::Active;
        
        env.storage()
            .instance()
            .set(&symbol_short!("lease"), &lease);
            
        symbol_short!("active")
    }
    
    /// Updates property metadata URI
    pub fn update_property_uri(env: Env, landlord: Address, property_uri: String) -> Symbol {
        let mut lease = Self::get_lease(env.clone());
        
        require!(lease.landlord == landlord, "Unauthorized: Only landlord can update property URI");
        
        lease.property_uri = property_uri.clone();
        
        env.storage()
            .instance()
            .set(&symbol_short!("lease"), &lease);
            
        symbol_short!("updated")
    }
    
    /// Amends lease with both landlord and tenant signatures
    pub fn amend_lease(env: Env, amendment: LeaseAmendment) -> Symbol {
        let mut lease = Self::get_lease(env.clone());
        
        require!(lease.status == LeaseStatus::Active, "Can only amend active leases");
        
        // In a real implementation, this would verify the signatures
        // For now, we'll assume they are valid
        
        if let Some(new_rent) = amendment.new_rent_amount {
            lease.rent_amount = new_rent;
        }
        
        if let Some(new_end_date) = amendment.new_end_date {
            lease.end_date = new_end_date;
        }
        
        env.storage()
            .instance()
            .set(&symbol_short!("lease"), &lease);
            
        symbol_short!("amended")
    }
    
    /// Releases security deposit with conditional logic
    pub fn release_deposit(env: Env, release_type: DepositRelease) -> Symbol {
        let lease = Self::get_lease(env.clone());
        
        require!(lease.status == LeaseStatus::Active || lease.status == LeaseStatus::Expired, 
                 "Can only release deposit from active or expired leases");
        
        match release_type {
            DepositRelease::FullRefund => {
                // In a real implementation, this would transfer full deposit to tenant
                symbol_short!("full_ref")
            }
            DepositRelease::PartialRefund(partial) => {
                require!(partial.tenant_amount + partial.landlord_amount == lease.deposit_amount, 
                         "Amounts must sum to total deposit");
                // In a real implementation, this would transfer amounts accordingly
                symbol_short!("partial")
            }
            DepositRelease::Disputed => {
                let mut updated_lease = lease;
                updated_lease.status = LeaseStatus::Disputed;
                env.storage()
                    .instance()
                    .set(&symbol_short!("lease"), &updated_lease);
                symbol_short!("disputed")
            }
        }
    }

    /// Returns the current lease details stored in the contract.
    pub fn get_lease(env: Env) -> Lease {
        env.storage()
            .instance()
            .get(&symbol_short!("lease"))
            .expect("Lease not found")
    }
}

mod test;
