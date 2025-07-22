
#![no_std]
#![allow(static_mut_refs)]

use sails_rs::{
    prelude::*,
    gstd::{msg, exec},
    collections::HashMap,
};
use extended_vft_client::vft::io as vft_io;
use sails_rs::calls::ActionIo;

// Imports required for session signless
use crate::{SessionData, Storage};

pub static mut CONTRACT_STATE: Option<ContractState> = None;

// This struct holds balances per user and total received by the contract
#[derive(Debug, Clone, Default)]
pub struct ContractState {
    pub user_balances: HashMap<ActorId, u128>,
    pub total_received: u128,
}

impl ContractState {
    pub fn init_state() {
        unsafe {
            CONTRACT_STATE = Some(Self::default());
        }
    }

    pub fn state_mut() -> &'static mut ContractState {
        let state = unsafe { CONTRACT_STATE.as_mut() };
        debug_assert!(state.is_some(), "The state is not initialized");
        unsafe { state.unwrap_unchecked() }
    }

    pub fn state_ref() -> &'static ContractState {
        let state = unsafe { CONTRACT_STATE.as_ref() };
        debug_assert!(state.is_some(), "The state is not initialized");
        unsafe { state.unwrap_unchecked() }
    }
}

// For session-based calls/actions
#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum ActionsForSession {
    Deposit,
}

// Used for program events
#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Events {
    Deposited(ActorId, u128),
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, Default)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct IoContractState {
    pub user_balances: Vec<(ActorId, u128)>,
    pub total_received: u128,
}

impl From<ContractState> for IoContractState {
    fn from(state: ContractState) -> Self {
        Self {
            user_balances: state.user_balances.iter().map(|(k, v)| (*k, *v)).collect(),
            total_received: state.total_received,
        }
    }
}

#[derive(Default)]
pub struct Service;

impl Service {
    pub fn seed() {
        ContractState::init_state();
    }
}

// Helper for session verification; returns correct actor for sender or session flow
fn get_actor(
    session_map: &HashMap<ActorId, SessionData>,
    msg_source: &ActorId,
    session_for_account: &Option<ActorId>,
    action: ActionsForSession,
) -> ActorId {
    match session_for_account {
        Some(account) => {
            let session = session_map
                .get(account)
                .expect("No valid session for this account");
            assert!(
                session.expires > exec::block_timestamp(),
                "Session expired"
            );
            assert!(
                session.allowed_actions.contains(&action),
                "Action not allowed"
            );
            assert_eq!(
                session.key, *msg_source,
                "Sender not authorized for session"
            );
            *account
        }
        None => *msg_source,
    }
}

#[sails_rs::service(events = Events)]
impl Service {
    pub fn new() -> Self {
        Self
    }

    /// Transfers FT tokens from a user to this contract with signless/session support.
   pub async fn deposit(
    &mut self,
    token_contract: ActorId,
    amount: u128,
    session_for_account: Option<ActorId>,
) -> Events {
    assert!(amount > 0, "Zero amount");
    let msg_src = msg::source();
    let sessions = Storage::get_session_map();
    let actor = get_actor(&sessions, &msg_src, &session_for_account, ActionsForSession::Deposit);

    let amount_u256 = U256::from(amount);

    let request = vft_io::TransferFrom::encode_call(actor, exec::program_id(), amount_u256);

    let send_result = msg::send_bytes_with_gas_for_reply(token_contract, request, 5_000_000_000, 0, 0);

    match send_result {
        Ok(reply_future) => match reply_future.await {
            Ok(reply) => {
               
            }
            Err(e) => {
                panic!("TransferFrom did not reply successfully: {:?}", e);
            }
        },
        Err(e) => {
            panic!("Send TransferFrom failed: {:?}", e);
        }
    }

    // Actualiza el estado para actor y contrato
    let state = ContractState::state_mut();
    let entry = state.user_balances.entry(actor).or_default();
    *entry = entry.saturating_add(amount);
    state.total_received = state.total_received.saturating_add(amount);

    self.emit_event(Events::Deposited(actor, amount)).expect("Event error");
    Events::Deposited(actor, amount)
}

    /// Returns the balance of a given user (query)
    pub fn query_user_balance(&self, user: ActorId) -> u128 {
        ContractState::state_ref().user_balances.get(&user).cloned().unwrap_or_default()
    }

    /// Returns the total amount received by the contract (query)
    pub fn query_total_received(&self) -> u128 {
        ContractState::state_ref().total_received
    }

    /// Returns the full state (query)
    pub fn query_state(&self) -> IoContractState {
        ContractState::state_ref().clone().into()
    }
}
