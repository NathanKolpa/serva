use crate::ipc::Endpoint;
use crate::router::Router;

struct RouterNode<'a, H> {
    parent: &'a StackRouter<'a, H>,
    match_on: Endpoint,
    handler: H,
}

pub struct StackRouter<'a, H> {
    node: Option<RouterNode<'a, H>>,
}

impl<'a, H> StackRouter<'a, H> {
    pub const fn new() -> Self {
        Self { node: None }
    }

    #[must_use]
    pub fn route<E: AsRef<str>>(&'a self, name: E, handler: H) -> Option<Self> {
        let endpoint = Endpoint::lookup(name)?;

        Some(Self {
            node: Some(RouterNode {
                match_on: endpoint,
                handler,
                parent: self,
            }),
        })
    }
}

impl<'a, H> Router<H> for StackRouter<'a, H>
where
    H: Copy,
{
    fn forward_to(&self, endpoint: Endpoint) -> Option<H> {
        let Some(node) = self.node.as_ref() else {
            return None;
        };

        if node.match_on == endpoint {
            return Some(node.handler);
        }

        return node.parent.forward_to(endpoint);
    }
}
