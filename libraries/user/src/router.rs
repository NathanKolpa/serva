mod stack_router;

use crate::ipc::Endpoint;
pub use stack_router::*;


pub trait Router<H> {
    fn forward_to(&self, endpoint: Endpoint) -> Option<H>;
}