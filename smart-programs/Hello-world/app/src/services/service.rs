#![allow(static_mut_refs)]

use sails_rs::{
    prelude::*,
    gstd::{msg, exec},
};
use sails_rs::collections::HashMap;

use crate::{SessionData, Storage}; 

pub static mut HELLO_STATE: Option<HelloState> = None;

#[derive(Clone, Default)]
pub struct HelloState {
    pub greeting: String,
    pub user_greetings: HashMap<ActorId, String>,
    pub counter: u64,
}

impl HelloState {
    pub fn init_state() {
        unsafe {
            HELLO_STATE = Some(Self {
                greeting: "Hello World from Vara Network!".to_string(),
                user_greetings: HashMap::new(),
                counter: 0,
            });
        }
    }

    pub fn state_mut() -> &'static mut HelloState {
        let state = unsafe { HELLO_STATE.as_mut() };
        debug_assert!(state.is_some(), "The state is not initialized");
        unsafe { state.unwrap_unchecked() }
    }

    pub fn state_ref() -> &'static HelloState {
        let state = unsafe { HELLO_STATE.as_ref() };
        debug_assert!(state.is_some(), "The state is not initialized");
        unsafe { state.unwrap_unchecked() }
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Events {
    Hello,
    PersonalHello(ActorId, String),
    GreetingSet(String),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct IoHelloState {
    pub greeting: String,
    pub user_greetings: Vec<(ActorId, String)>,
    pub counter: u64,
}

impl From<HelloState> for IoHelloState {
    fn from(state: HelloState) -> Self {
        Self {
            greeting: state.greeting,
            user_greetings: state
                .user_greetings
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
            counter: state.counter,
        }
    }
}

#[derive(Default)]
pub struct Service;

impl Service {
    pub fn seed() {
        HelloState::init_state();
    }
}

#[sails_rs::service(events = Events)]
impl Service {
    pub fn new() -> Self {
        Self
    }

    pub fn hello_world(&mut self, session_for_account: Option<ActorId>) -> Events {
        let msg_src = msg::source();
        let sessions = Storage::get_session_map();
        let _actor = get_actor(&sessions, &msg_src, &session_for_account, ActionsForSession::SayHello);

        HelloState::state_mut().counter += 1;
        self.emit_event(Events::Hello).expect("Notification failure");
        Events::Hello
    }

    pub fn personal_hello(&mut self, name: String, session_for_account: Option<ActorId>) -> Events {
        let msg_src = msg::source();
        let sessions = Storage::get_session_map();
        let actor = get_actor(&sessions, &msg_src, &session_for_account, ActionsForSession::SayPersonalHello);

        let message = format!("Hello {} from Vara Network!", name);
        let state = HelloState::state_mut();
        state.user_greetings.insert(actor, message.clone());
        state.counter += 1;

        self.emit_event(Events::PersonalHello(actor, name.clone())).expect("Notification failure");
        Events::PersonalHello(actor, name)
    }

    pub fn set_greeting(&mut self, new_greeting: String, session_for_account: Option<ActorId>) -> Events {
        let msg_src = msg::source();
        let sessions = Storage::get_session_map();
        let _actor = get_actor(&sessions, &msg_src, &session_for_account, ActionsForSession::SetGreeting);

        HelloState::state_mut().greeting = new_greeting.clone();
        self.emit_event(Events::GreetingSet(new_greeting.clone())).expect("Notification failure");
        Events::GreetingSet(new_greeting)
    }

    pub fn query_greeting(&self) -> String {
        HelloState::state_ref().greeting.clone()
    }

    pub fn query_user_greeting(&self, user: ActorId) -> Option<String> {
        HelloState::state_ref().user_greetings.get(&user).cloned()
    }

    pub fn query_counter(&self) -> u64 {
        HelloState::state_ref().counter
    }

    pub fn query_state(&self) -> IoHelloState {
        HelloState::state_ref().clone().into()
    }
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum ActionsForSession {
    SayHello,
    SayPersonalHello,
    SetGreeting,
}

fn get_actor(
    session_map: &HashMap<ActorId, SessionData>,
    msg_source: &ActorId,
    session_for_account: &Option<ActorId>,
    actions_for_session: ActionsForSession,
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
                session.allowed_actions.contains(&actions_for_session),
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