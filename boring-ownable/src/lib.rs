#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;

use contract::{contract_api::{runtime, storage}, unwrap_or_revert::UnwrapOrRevert};
use std::ops::{Add, Sub};
use core::convert::TryInto;
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter},
    ApiError, CLType, CLTyped, Key, URef,system::CallStackElement, CLValue
};

#[repr(u16)]
pub enum Error {
    BoringOwnableZeroAddress = 0,
    BoringOwnableForbidden = 1,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

const OWNER_KEY: &str = "OWNER_KEY";
const PENDING_OWNER_KEY: &str = "PENDING_OWNER_KEY";

/* ✖✖✖✖✖✖✖✖✖✖✖ Public getters - Start ✖✖✖✖✖✖✖✖✖✖✖ */
#[no_mangle]
pub extern "C" fn owner() {
    let val: AccountHash = get_key(&OWNER_KEY);
    ret(val)
}

#[no_mangle]
pub extern "C" fn pending_owner() {
    let val: AccountHash = get_key(&PENDING_OWNER_KEY);
    ret(val)
}
/* ✖✖✖✖✖✖✖✖✖✖✖ Public getters - End ✖✖✖✖✖✖✖✖✖✖✖ */

/* ✖✖✖✖✖✖✖✖✖✖✖ Public functions - Start ✖✖✖✖✖✖✖✖✖✖✖ */
#[no_mangle]
pub extern "C" fn transfer_ownership() {
    _only_owner();
    let new_owner: AccountHash = runtime::get_named_arg("new_owner");
    let direct: bool = runtime::get_named_arg("direct");
    let renounce: bool = runtime::get_named_arg("renounce");

    _transfer_ownership(new_owner, direct, renounce)
}

#[no_mangle]
pub extern "C" fn claim_ownership() {
    _only_pending_owner();

    _claim_ownership()
}
/* ✖✖✖✖✖✖✖✖✖✖✖ Public functions - End ✖✖✖✖✖✖✖✖✖✖✖ */

// All session code must have a `call` entrypoint.
#[no_mangle]
pub extern "C" fn call() {
    // Get the optional first argument supplied to the argument.
    let owner: AccountHash = runtime::get_named_arg("owner");

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(endpoint("owner", vec![], AccountHash::cl_type()));
    entry_points.add_entry_point(endpoint("pending_owner", vec![], AccountHash::cl_type()));

    set_key(OWNER_KEY, owner);
}

/* ✖✖✖✖✖✖✖✖✖✖✖ Internal Functions - Start ✖✖✖✖✖✖✖✖✖✖✖ */
fn _transfer_ownership(new_owner: AccountHash, direct: bool, renounce: bool) {
    if direct {
        if new_owner != AccountHash::default() || renounce {
            set_key(&OWNER_KEY, new_owner);
        } else {
            runtime::revert(Error::BoringOwnableZeroAddress);
        }
    } else {
        set_key(&PENDING_OWNER_KEY, new_owner);
    }
}

fn _claim_ownership() {
    let pending_owner: AccountHash = get_key(&PENDING_OWNER_KEY);
    set_key(&OWNER_KEY, pending_owner);
    set_key(&PENDING_OWNER_KEY, AccountHash::default());
}

fn _only_owner() {
    if (
        get_caller() != 
        Key::Account(get_key::<AccountHash>(&OWNER_KEY))
    ) {
        runtime::revert(Error::BoringOwnableForbidden);
    }
}

fn _only_pending_owner() {
    if (
        get_caller() != 
        Key::Account(get_key::<AccountHash>(&PENDING_OWNER_KEY))
    ) {
        runtime::revert(Error::BoringOwnableForbidden);
    }
}
/* ✖✖✖✖✖✖✖✖✖✖✖ Internal Functions - End ✖✖✖✖✖✖✖✖✖✖✖ */

fn ret<T: CLTyped + ToBytes>(value: T) {
    runtime::ret(CLValue::from_t(value).unwrap_or_revert())
}

fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

fn endpoint(name: &str, param: Vec<Parameter>, ret: CLType) -> EntryPoint {
    EntryPoint::new(
        String::from(name),
        param,
        ret,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn get_caller() -> Key {
    let mut callstack = runtime::get_call_stack();
    callstack.pop();
    match callstack.last().unwrap_or_revert() {
        CallStackElement::Session { account_hash } => (*account_hash).into(),
        CallStackElement::StoredSession {
            account_hash,
            contract_package_hash: _,
            contract_hash: _,
        } => (*account_hash).into(),
        CallStackElement::StoredContract {
            contract_package_hash: _,
            contract_hash,
        } => (*contract_hash).into(),
    }
}
