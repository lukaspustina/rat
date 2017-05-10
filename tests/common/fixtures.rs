use std::panic;

#[derive(Debug)]
pub struct EmptyContext {}

impl EmptyContext {
    pub fn new() -> Self {
        EmptyContext {}
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Context<T> {
    pub ctx: Option<T>
}

#[allow(dead_code)]
impl<T> Context<T> {
    pub fn new() -> Self {
        Context { ctx: None }
    }
    pub fn ctx(&self) -> &T { self.ctx.as_ref().unwrap() }
}

pub trait Fixture {
    fn setup(&mut self) -> () {}

    fn run_test<T>(&mut self, test: T) -> () where T: FnOnce(&Self) -> () + panic::UnwindSafe, Self: panic::RefUnwindSafe {
        self.setup();

        let result = panic::catch_unwind(|| {
            test(self)
        });

        self.teardown();

        assert!(result.is_ok())
    }

    fn teardown(&mut self) -> () {}
}
