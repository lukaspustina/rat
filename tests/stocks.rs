extern crate rat;
#[macro_use] extern crate spectral;

mod common;

mod stocks {
    mod comdirect {
        use ::common::fixtures::*;

        use rat;
        use rat::modules::stocks::comdirect::scrape_stock_price;
        use rat::modules::stocks::comdirect::{Error, ErrorKind};

        use spectral::prelude::*;

        impl Fixture for EmptyContext {
            fn setup(&mut self) {
                rat::utils::console::init(rat::config::Verbosity::QUIET);
            }
        }

        #[test]
        fn successfully_retrieve_deutsche_bank() {
            EmptyContext::new().run_test(|_| {
                let db = scrape_stock_price("Deutsche Bank".to_string()).unwrap();

                assert_that!(db.name).named("Name").is_equal_to("Deutsche Bank AG Namens-Aktien o.N.".to_owned());
                assert_that!(db.wkn).named("WKN").is_equal_to("514000".to_owned());
                assert_that!(db.price).named("price").is_greater_than(0.00f32);
                assert_that!(db.currency).named("Currency").is_equal_to("EUR".to_owned());
            })
        }

        #[test]
        fn stock_search_returns_multiple_results() {
            EmptyContext::new().run_test(|_| {
                rat::utils::console::init(rat::config::Verbosity::QUIET);
                let result = scrape_stock_price("Deutsche".to_string());

                // TODO: This needs to be nicer.
                let result_is_not_unique = match result.unwrap_err() {
                    Error(ErrorKind::ComdirectSearchResultNotUnique, _) => true,
                    _ => false
                };
                assert_that!(result_is_not_unique).named("ComdirectSearchResultNotUnique").is_true();
            })
        }
    }
}