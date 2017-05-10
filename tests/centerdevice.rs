extern crate rat;
#[macro_use] extern crate spectral;

mod common;

mod centerdevice {
    mod status {
        use ::common::fixtures::*;

        use rat;
        use rat::modules::centerdevice::status;

        use spectral::prelude::*;

        impl Fixture for EmptyContext {
            fn setup(&mut self) {
                rat::utils::console::init(rat::config::Verbosity::QUIET);
            }
        }

        #[test]
        fn status() {
            EmptyContext::new().run_test(|_| {
                rat::utils::console::init(rat::config::Verbosity::QUIET);
                let result = status::get_centerdevice_status_json();

                assert_that!(result).named("CenterDevice Status").is_ok();
            })
        }
    }
}