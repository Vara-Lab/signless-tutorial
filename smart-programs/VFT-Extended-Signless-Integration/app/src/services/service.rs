
#![no_std]
#![allow(static_mut_refs)]

use gstd::{msg, exec};
use sails_rs::{collections::HashSet, gstd::service, prelude::*};
use vft_service::utils;
use vft_service::{Service as VftService, Storage};
use vft_service::{
    funcs,
    utils::{Error, Result, *},
};

pub mod session_service {
    use super::*;
    use sails_rs::collections::HashMap;

    #[derive(Clone, Encode, Decode, TypeInfo, Default)]
    #[codec(crate = sails_rs::scale_codec)]
    #[scale_info(crate = sails_rs::scale_info)]
    pub struct SessionData {
        pub key: ActorId,
        pub expires: u64,
        pub allowed_actions: Vec<ActionsForSession>,
    }

    #[derive(Default)]
    pub struct SessionStorage {
        pub session_map: HashMap<ActorId, SessionData>,
    }

    static mut SESSION_STORAGE: Option<SessionStorage> = None;

    impl SessionStorage {
        pub fn get_mut() -> &'static mut SessionStorage {
            unsafe {
                SESSION_STORAGE
                    .as_mut()
                    .expect("Session storage not initialized")
            }
        }
        pub fn get() -> &'static SessionStorage {
            unsafe {
                SESSION_STORAGE
                    .as_ref()
                    .expect("Session storage not initialized")
            }
        }
        pub fn seed() {
            unsafe {
                SESSION_STORAGE = Some(SessionStorage {
                    session_map: HashMap::new(),
                });
            }
        }
    }
    pub fn get_session_map() -> &'static HashMap<ActorId, SessionData> {
        &SessionStorage::get().session_map
    }
}

use session_service::{SessionData, get_session_map};

#[derive(Default)]
pub struct ExtendedStorage {
    minters: HashSet<ActorId>,
    burners: HashSet<ActorId>,
    admins: HashSet<ActorId>,
}

static mut EXTENDED_STORAGE: Option<ExtendedStorage> = None;

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Event {
    Minted { to: ActorId, value: U256 },
    Burned { from: ActorId, value: U256 },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum ActionsForSession {
    Mint,
    Burn,
    Approve,
    GrantAdmin,
    GrantMinter,
    GrantBurner,
    RevokeAdmin,
    RevokeMinter,
    RevokeBurner,
}


#[derive(Clone)]
pub struct ExtendedService {
    vft: VftService,
}

impl ExtendedService {
  
    pub fn seed(name: String, symbol: String, decimals: u8) -> Self {
        let admin = msg::source();
        unsafe {
            EXTENDED_STORAGE = Some(ExtendedStorage {
                admins: [admin].into(),
                minters: [admin].into(),
                burners: [admin].into(),
            });
        };
        session_service::SessionStorage::seed();
        ExtendedService {
            vft: <VftService>::seed(name, symbol, decimals),
        }
    }

 
    pub fn get_mut(&mut self) -> &'static mut ExtendedStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_mut()
                .expect("Extended vft is not initialized")
        }
    }
   

    pub fn get(&self) -> &'static ExtendedStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_ref()
                .expect("Extended vft is not initialized")
        }
    }
}


fn get_actor(
    session_map: &sails_rs::collections::HashMap<ActorId, SessionData>,
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


#[service(extends = VftService, events = Event)]
impl ExtendedService {

    pub fn new() -> Self {
        Self {
            vft: VftService::new(),
        }
    }

   pub fn approve(&mut self, spender: ActorId, value: U256, session_for_account: Option<ActorId>) {
    let msg_src = msg::source();
    let owner = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::Approve);

    let ok = self.vft.approve(spender, value); 
    assert!(ok, "Approval failed");
}



    pub fn mint(&mut self, to: ActorId, value: U256, session_for_account: Option<ActorId>) -> bool {
        let msg_src = msg::source();
        let actor = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::Mint);

        if !self.get().minters.contains(&actor) {
            panic!("Not allowed to mint")
        };

        let mutated = utils::panicking(|| {
             mint(Storage::balances(), Storage::total_supply(), to, value)
        });
        if mutated {
            self.emit_event(Event::Minted { to, value })
                .expect("Notification Error");
        }
        mutated
    }


    pub fn burn(&mut self, from: ActorId, value: U256, session_for_account: Option<ActorId>) -> bool {
        let msg_src = msg::source();
        let actor = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::Burn);

        if !self.get().burners.contains(&actor) {
            panic!("Not allowed to burn")
        };

        let mutated = utils::panicking(|| {
            burn(Storage::balances(), Storage::total_supply(), from, value)
        });
        if mutated {
            self.emit_event(Event::Burned { from, value })
                .expect("Notification Error");
        }
        mutated
    }

    pub fn grant_admin_role(&mut self, to: ActorId, session_for_account: Option<ActorId>) {
        let msg_src = msg::source();
        let actor = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::GrantAdmin);
        self.ensure_is_admin_actor(actor);
        self.get_mut().admins.insert(to);
    }
  

    pub fn grant_minter_role(&mut self, to: ActorId, session_for_account: Option<ActorId>) {
        let msg_src = msg::source();
        let actor = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::GrantMinter);
        self.ensure_is_admin_actor(actor);
        self.get_mut().minters.insert(to);
    }


    pub fn grant_burner_role(&mut self, to: ActorId, session_for_account: Option<ActorId>) {
        let msg_src = msg::source();
        let actor = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::GrantBurner);
        self.ensure_is_admin_actor(actor);
        self.get_mut().burners.insert(to);
    }


    pub fn revoke_admin_role(&mut self, from: ActorId, session_for_account: Option<ActorId>) {
        let msg_src = msg::source();
        let actor = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::RevokeAdmin);
        self.ensure_is_admin_actor(actor);
        self.get_mut().admins.remove(&from);
    }


    pub fn revoke_minter_role(&mut self, from: ActorId, session_for_account: Option<ActorId>) {
        let msg_src = msg::source();
        let actor = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::RevokeMinter);
        self.ensure_is_admin_actor(actor);
        self.get_mut().minters.remove(&from);
    }


    pub fn revoke_burner_role(&mut self, from: ActorId, session_for_account: Option<ActorId>) {
        let msg_src = msg::source();
        let actor = get_actor(get_session_map(), &msg_src, &session_for_account, ActionsForSession::RevokeBurner);
        self.ensure_is_admin_actor(actor);
        self.get_mut().burners.remove(&from);
    }

 
    pub fn minters(&self) -> Vec<ActorId> {
        self.get().minters.clone().into_iter().collect()
    }


    pub fn burners(&self) -> Vec<ActorId> {
        self.get().burners.clone().into_iter().collect()
    }

    pub fn admins(&self) -> Vec<ActorId> {
        self.get().admins.clone().into_iter().collect()
    }
}

impl ExtendedService {

    fn ensure_is_admin(&self) {
        if !self.get().admins.contains(&msg::source()) {
            panic!("Not admin")
        };
    }
    fn ensure_is_admin_actor(&self, actor: ActorId) {
        if !self.get().admins.contains(&actor) {
            panic!("Not admin")
        };
    }
}


impl AsRef<VftService> for ExtendedService {
    fn as_ref(&self) -> &VftService {
        &self.vft
    }
}

pub fn mint(
    balances: &mut BalancesMap,
    total_supply: &mut U256,
    to: ActorId,
    value: U256,
) -> Result<bool> {
    if value.is_zero() {
        return Ok(false);
    }

    let new_total_supply = total_supply
        .checked_add(value)
        .ok_or(Error::NumericOverflow)?;

    let new_to = funcs::balance_of(balances, to)
        .checked_add(value)
        .ok_or(Error::NumericOverflow)?;

    balances.insert(to, new_to);
    *total_supply = new_total_supply;

    Ok(true)
}

pub fn burn(
    balances: &mut BalancesMap,
    total_supply: &mut U256,
    from: ActorId,
    value: U256,
) -> Result<bool> {
    if value.is_zero() {
        return Ok(false);
    }
    let new_total_supply = total_supply.checked_sub(value).ok_or(Error::Underflow)?;

    let new_from = funcs::balance_of(balances, from)
        .checked_sub(value)
        .ok_or(Error::InsufficientBalance)?;

    if !new_from.is_zero() {
        balances.insert(from, new_from);
    } else {
        balances.remove(&from);
    }

    *total_supply = new_total_supply;
    Ok(true)
}
