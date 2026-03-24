#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, BytesN, String};
use crate::LeaseContractClient;

#[test]
fn test_lease_initialization() {
    let env = Env::default();
    let contract_id = env.register(LeaseContract, ());
    let client = LeaseContractClient::new(&env, &contract_id);

    let landlord = Address::generate(&env);
    let tenant = Address::generate(&env);
    let rent_amount = 1000i128;
    let deposit_amount = 2000i128;
    let start_date = 1640995200u64; // Jan 1, 2022
    let end_date = 1672531200u64; // Jan 1, 2023
    let property_uri = String::from_str(&env, "ipfs://QmHash123");

    client.initialize_lease(
        &landlord,
        &tenant,
        &rent_amount,
        &deposit_amount,
        &start_date,
        &end_date,
        &property_uri,
    );
    
    let lease = client.get_lease();
    assert_eq!(lease.landlord, landlord);
    assert_eq!(lease.tenant, tenant);
    assert_eq!(lease.rent_amount, rent_amount);
    assert_eq!(lease.deposit_amount, deposit_amount);
    assert_eq!(lease.start_date, start_date);
    assert_eq!(lease.end_date, end_date);
    assert_eq!(lease.property_uri, property_uri);
    assert_eq!(lease.status, LeaseStatus::Pending);
}

#[test]
fn test_lease_activation() {
    let env = Env::default();
    let contract_id = env.register(LeaseContract, ());
    let client = LeaseContractClient::new(&env, &contract_id);

    let landlord = Address::generate(&env);
    let tenant = Address::generate(&env);
    let rent_amount = 1000i128;
    let deposit_amount = 2000i128;
    let start_date = 1640995200u64;
    let end_date = 1672531200u64;
    let property_uri = String::from_str(&env, "ipfs://QmHash123");

    client.initialize_lease(
        &landlord,
        &tenant,
        &rent_amount,
        &deposit_amount,
        &start_date,
        &end_date,
        &property_uri,
    );
    
    // Activate lease
    client.activate_lease(&tenant);
    
    let lease = client.get_lease();
    assert_eq!(lease.status, LeaseStatus::Active);
}

#[test]
fn test_property_uri_update() {
    let env = Env::default();
    let contract_id = env.register(LeaseContract, ());
    let client = LeaseContractClient::new(&env, &contract_id);

    let landlord = Address::generate(&env);
    let tenant = Address::generate(&env);
    let rent_amount = 1000i128;
    let deposit_amount = 2000i128;
    let start_date = 1640995200u64;
    let end_date = 1672531200u64;
    let property_uri = String::from_str(&env, "ipfs://QmHash123");

    client.initialize_lease(
        &landlord,
        &tenant,
        &rent_amount,
        &deposit_amount,
        &start_date,
        &end_date,
        &property_uri,
    );
    
    // Update property URI
    let new_property_uri = String::from_str(&env, "ipfs://QmNewHash456");
    client.update_property_uri(&landlord, &new_property_uri);
    
    let lease = client.get_lease();
    assert_eq!(lease.property_uri, new_property_uri);
}

#[test]
fn test_lease_amendment() {
    let env = Env::default();
    let contract_id = env.register(LeaseContract, ());
    let client = LeaseContractClient::new(&env, &contract_id);

    let landlord = Address::generate(&env);
    let tenant = Address::generate(&env);
    let rent_amount = 1000i128;
    let deposit_amount = 2000i128;
    let start_date = 1640995200u64;
    let end_date = 1672531200u64;
    let property_uri = String::from_str(&env, "ipfs://QmHash123");

    client.initialize_lease(
        &landlord,
        &tenant,
        &rent_amount,
        &deposit_amount,
        &start_date,
        &end_date,
        &property_uri,
    );
    
    client.activate_lease(&tenant);
    
    // Create amendment with new rent and end date
    let new_rent = Some(1200i128);
    let new_end_date = Some(1704067200u64); // Jan 1, 2024
    let landlord_sig = BytesN::from_array(&env, &[1u8; 32]);
    let tenant_sig = BytesN::from_array(&env, &[2u8; 32]);
    
    let amendment = LeaseAmendment {
        new_rent_amount: new_rent,
        new_end_date: new_end_date,
        landlord_signature: landlord_sig,
        tenant_signature: tenant_sig,
    };
    
    client.amend_lease(&amendment);
    
    let lease = client.get_lease();
    assert_eq!(lease.rent_amount, 1200i128);
    assert_eq!(lease.end_date, 1704067200u64);
}

#[test]
fn test_deposit_release_full_refund() {
    let env = Env::default();
    let contract_id = env.register(LeaseContract, ());
    let client = LeaseContractClient::new(&env, &contract_id);

    let landlord = Address::generate(&env);
    let tenant = Address::generate(&env);
    let rent_amount = 1000i128;
    let deposit_amount = 2000i128;
    let start_date = 1640995200u64;
    let end_date = 1672531200u64;
    let property_uri = String::from_str(&env, "ipfs://QmHash123");

    client.initialize_lease(
        &landlord,
        &tenant,
        &rent_amount,
        &deposit_amount,
        &start_date,
        &end_date,
        &property_uri,
    );
    
    client.activate_lease(&tenant);
    
    // Release full deposit
    let release = DepositRelease::FullRefund;
    client.release_deposit(&release);
}

#[test]
fn test_deposit_release_partial_refund() {
    let env = Env::default();
    let contract_id = env.register(LeaseContract, ());
    let client = LeaseContractClient::new(&env, &contract_id);

    let landlord = Address::generate(&env);
    let tenant = Address::generate(&env);
    let rent_amount = 1000i128;
    let deposit_amount = 2000i128;
    let start_date = 1640995200u64;
    let end_date = 1672531200u64;
    let property_uri = String::from_str(&env, "ipfs://QmHash123");

    client.initialize_lease(
        &landlord,
        &tenant,
        &rent_amount,
        &deposit_amount,
        &start_date,
        &end_date,
        &property_uri,
    );
    
    client.activate_lease(&tenant);
    
    // Release partial deposit
    let partial = DepositReleasePartial {
        tenant_amount: 1500i128,
        landlord_amount: 500i128,
    };
    let release = DepositRelease::PartialRefund(partial);
    client.release_deposit(&release);
}

#[test]
fn test_deposit_release_disputed() {
    let env = Env::default();
    let contract_id = env.register(LeaseContract, ());
    let client = LeaseContractClient::new(&env, &contract_id);

    let landlord = Address::generate(&env);
    let tenant = Address::generate(&env);
    let rent_amount = 1000i128;
    let deposit_amount = 2000i128;
    let start_date = 1640995200u64;
    let end_date = 1672531200u64;
    let property_uri = String::from_str(&env, "ipfs://QmHash123");

    client.initialize_lease(
        &landlord,
        &tenant,
        &rent_amount,
        &deposit_amount,
        &start_date,
        &end_date,
        &property_uri,
    );
    
    client.activate_lease(&tenant);
    
    // Mark deposit as disputed
    let release = DepositRelease::Disputed;
    client.release_deposit(&release);
    
    let lease = client.get_lease();
    assert_eq!(lease.status, LeaseStatus::Disputed);
}
