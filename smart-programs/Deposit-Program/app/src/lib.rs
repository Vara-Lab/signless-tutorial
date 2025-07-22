
#![no_std]
use sails_rs::prelude::*;
pub mod services;
use services::service::{Service, ActionsForSession};
use session_service::*;

session_service::generate_session_system!(ActionsForSession);

pub struct Program;

#[program]
impl Program {
    pub fn new(config: Config) -> Self {
        Service::seed();
        SessionService::init(config);
        Self
    }

    #[route("Service")]
    pub fn service(&self) -> Service {
        Service::new()
    }

    #[route("Session")]
    pub fn session(&self) -> SessionService {
        SessionService::new()
    }
}
