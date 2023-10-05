use crate::ipc::{Endpoint, Request};
use core::marker::PhantomData;

const EXPECT_HANDLER: &str = "endpoint should have a handler specified";

pub trait RouterMut {
    fn try_forward_to_mut(&mut self, endpoint: &Endpoint) -> Option<&mut dyn FnMut(Request)>;

    fn forward_to_mut(&mut self, endpoint: &Endpoint) -> &mut dyn FnMut(Request) {
        self.try_forward_to_mut(endpoint).expect(EXPECT_HANDLER)
    }
}

pub trait Router {
    fn try_forward_to(&self, endpoint: &Endpoint) -> Option<&dyn Fn(Request)>;

    fn forward_to(&self, endpoint: &Endpoint) -> &dyn Fn(Request) {
        self.try_forward_to(endpoint).expect(EXPECT_HANDLER)
    }
}

pub struct StackRouter<Handler, Endpoint, Parent> {
    parent: Parent,
    match_on: Endpoint,
    handler: Handler,
}

impl StackRouter<PhantomData<()>, PhantomData<()>, PhantomData<()>> {
    pub fn new() -> Self {
        Self {
            handler: PhantomData,
            match_on: PhantomData,
            parent: PhantomData,
        }
    }
}

impl<Handler, E, Parent> StackRouter<Handler, E, Parent> {
    #[must_use]
    pub fn route<N: AsRef<str>, NewHandler>(
        self,
        name: N,
        handler: NewHandler,
    ) -> StackRouter<NewHandler, Endpoint, Self>
    where
        NewHandler: FnMut(Request),
    {
        self.try_route(name, handler)
            .expect("endpoint name should exists in the current service's spec")
    }

    #[must_use]
    pub fn try_route<N: AsRef<str>, NewHandler>(
        self,
        name: N,
        handler: NewHandler,
    ) -> Option<StackRouter<NewHandler, Endpoint, Self>>
    where
        NewHandler: FnMut(Request),
    {
        let endpoint = Endpoint::lookup(name)?;

        Some(StackRouter {
            match_on: endpoint,
            handler,
            parent: self,
        })
    }
}

impl Router for StackRouter<PhantomData<()>, PhantomData<()>, PhantomData<()>> {
    fn try_forward_to(&self, _endpoint: &Endpoint) -> Option<&dyn Fn(Request)> {
        None
    }
}

impl RouterMut for StackRouter<PhantomData<()>, PhantomData<()>, PhantomData<()>> {
    fn try_forward_to_mut(&mut self, _endpoint: &Endpoint) -> Option<&mut dyn FnMut(Request)> {
        None
    }
}

impl<Handler, Parent> Router for StackRouter<Handler, Endpoint, Parent>
where
    Handler: Fn(Request),
    Parent: Router,
{
    fn try_forward_to(&self, endpoint: &Endpoint) -> Option<&dyn Fn(Request)> {
        if self.match_on == *endpoint {
            return Some(&self.handler);
        }

        return self.parent.try_forward_to(endpoint);
    }
}

impl<Handler, Parent> RouterMut for StackRouter<Handler, Endpoint, Parent>
where
    Handler: FnMut(Request),
    Parent: RouterMut,
{
    fn try_forward_to_mut(&mut self, endpoint: &Endpoint) -> Option<&mut dyn FnMut(Request)> {
        if self.match_on == *endpoint {
            return Some(&mut self.handler);
        }

        return self.parent.try_forward_to_mut(endpoint);
    }
}
