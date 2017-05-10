extern crate rat;
#[macro_use]
extern crate spectral;

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
                let result = status::get_centerdevice_status_json();

                assert_that!(result).named("CenterDevice Status").is_ok();
            })
        }
    }

    use common::fixtures::*;

    use rat;
    use rat::config::Config;

    use std::env;
    use std::path::Path;

    impl Fixture for Context<Config> {
        fn setup(&mut self) {
            rat::utils::console::init(rat::config::Verbosity::QUIET);

            let default_config_path = format!("{}/.{}", env::home_dir().unwrap().display(), "rat.toml");
            let config_path = Path::new(&default_config_path);
            let config = Config::from_file(config_path).unwrap();

            self.ctx = Some(config);
        }
    }

    mod search {
        use rat::modules::centerdevice::client;

        use super::*;
        use spectral::prelude::*;

        #[test]
        #[ignore]
        fn search_document() {
            let mut context: Context<Config> = Context::new();
            context.run_test(|ctx| {
                let config: &Config = ctx.ctx();
                assert_that!(config.centerdevice.access_token).named("CenterDevice Access Token").is_some();

                let result = client::search::search_documents(
                    config.centerdevice.access_token.as_ref().unwrap(),
                    None,
                    None,
                    Some("RAT integration test search"),
                    client::search::NamedSearches::None
                );

                assert_that!(result).named("CenterDevice Search").is_ok();
            })
        }
    }
}