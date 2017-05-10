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
pub struct Context<'a, T: 'a> {
    ctx: &'a T
}

#[allow(dead_code)]
impl<'a, T> Context<'a, T> {
    pub fn new(ctx: &'a T) -> Self {
        Context { ctx: ctx }
    }
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
