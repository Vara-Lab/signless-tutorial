#![no_std]

use sails_rs::prelude::*;
use session_service::*; 

mod services;
use services::service::{Service, ActionsForSession}; 

session_service::generate_session_system!(ActionsForSession);  

pub struct Program;

#[program]
impl Program {
    pub fn new(config: Config) -> Self {
        Service::seed();
        SessionService::init(config);
        Self
    }

    #[export(route = "Service")]
    pub fn service(&self) -> Service {
        Service::new()
    }

    #[export(route = "Session")]
    pub fn session(&self) -> SessionService {
        SessionService::new()
    }
}