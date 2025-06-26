

#![no_std]
use sails_rs::prelude::*;
pub mod services;
use services::service::{ActionsForSession, ExtendedService};
use session_service::*;


session_service::generate_session_system!(ActionsForSession);

pub struct Program;

#[program]
impl Program {
    pub fn new(name: String, symbol: String, decimals: u8, config: Config) -> Self {
        ExtendedService::seed(name, symbol, decimals); 
        SessionService::init(config);                         
        Self
    }

     #[route("Service")]
    pub fn service(&self) -> ExtendedService {
        ExtendedService::new()
    }

    #[route("Session")]
    pub fn session(&self) -> SessionService {
        SessionService::new()
    }
}