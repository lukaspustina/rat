use super::StockPrice;
use net::http::tls_client;

use hyper::header::Connection;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Element, Name, Predicate, Text};
use serde_urlencoded;
use std::io::Read;

static BASE_URL: &'static str = "https://www.comdirect.de/inf/search/all.html";

error_chain! {
    errors {
        ComdirectScrapingFailed(info: String) {
            description("Did not find information in HTML document")
            display("Did not find {} in HTML document", info)
        }
        ComdirectSearchResultNotUnique {
            description("The search did not returned an unique result")
            display("The search did not returned an unique result")
        }
    }
}

pub fn scrape_stock_price<T: FnMut(String) -> ()>(query: &str, mut progress: Option<T>) -> Result<StockPrice> {
    let parameters = &[("SEARCH_VALUE", query.to_owned())];
    let parameters_enc = serde_urlencoded::to_string(&parameters)
        .chain_err(|| "Could not encode URL parameters")?;
    let url = format!("{}?{}", BASE_URL, parameters_enc);

    if let Some(p) = progress.as_mut() {
        p("Sending search request ...".to_owned());
    }

    let body = get_stock_page(&url).unwrap();

    if let Some(p) = progress.as_mut() {
        p(format!("Received {} bytes", body.len()));
    }

    let stock_price = parse_stock_price(&body)?;

    Ok(stock_price)
}

fn get_stock_page(url: &str) -> Result<Vec<u8>> {
    let client = tls_client().chain_err(|| "Could not create TLS client")?;
    let mut response = client.get(url).header(Connection::close()).send()
        .chain_err(|| "Could not send request")?;

    let mut body = Vec::new();
    let _ = response.read_to_end(&mut body)
        .chain_err(|| "Could not read response body")?;

    Ok(body)
}

// /html/body/div[4]/div/div[1]/div[2]/div[1]/div/div/div[1]/div/span[2]
// body > div.cif-scope-content-wrapper.siteFrame > div > div.key-focus > div:nth-child(2) > div.col-3.col--sm-4 > div > div > div:nth-child(1) > div > span.realtime-indicator--value.text-size--xxlarge.text-weight--medium
fn parse_stock_price(body: &[u8]) -> Result<StockPrice> {
    let document = Document::from_read(body).chain_err(|| "Could not parse HTML in response body")?;

    let no_exact_match = document.find(Name("h1")).nth(0);
    if let Some(heading) = no_exact_match {
        if heading.text() == "Wertpapiersuche und Kursabfrage" {
            bail!(ErrorKind::ComdirectSearchResultNotUnique);
        }
    }

    let name = document.find(Name("h1")).nth(0)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("name".to_string()))?.text();

    let price_str = document.find(Class("key-focus__quote").descendant(Class("realtime-indicator").descendant(Text))).nth(0)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock price".to_string()))?
        .text()
        .trim()
        .replace(".", "") // thousand separation
        .replace(",", "."); // decimal separation
    let price = price_str
        .parse()
        .chain_err(|| ErrorKind::ComdirectScrapingFailed("parsing stock price".to_string()))?;

    let currency = document.find(Class("realtime-indicator").descendant(Text)).nth(1)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock currency".to_string()))?.text();

    let date_fn = |node: &Node| node.text() == "Stand";
    let date_text = document.find(Class("key-focus").descendant(date_fn)).nth(0)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock price date (1)".to_string()))?
        .parent()
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock price date (2)".to_string()))?
        .find(Element).nth(1)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock price date (3)".to_string()))?
        .first_child()
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock price date (4)".to_string()))?
        .text();
    let date = date_text
        .split('-')
        .nth(0)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock price date (5)".to_string()))?
        .trim();

    let wkn = document.find(Class("key-focus__info")).nth(0)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock WKN (1)".to_string()))?
        .last_child()
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock WKN (2)".to_string()))?
        .text();

    let stock_price = StockPrice {
        name: name.trim().to_string(),
        wkn: wkn.trim().to_string(),
        date: date.trim().to_string(),
        price,
        currency: currency.trim().to_string(),
    };

    Ok(stock_price)
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::Read;
    use std::fs::File;

    fn get_stock_page(file: &str) -> Result<Vec<u8>> {
        let mut f = File::open(file).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();

        Ok(buffer)
    }

    #[test]
    fn test_parse_file_ok() {
        let body = get_stock_page("test/data/stocks/deutsche_bank.html").unwrap();
        let db = parse_stock_price(&body).unwrap();

        assert_eq!(db.name, "DEUTSCHE BANK");
        assert_eq!(db.wkn, "514000");
        assert_eq!(db.date, "27.10.17");
        assert_eq!(db.price, 14.16f32);
        assert_eq!(db.currency, "EUR");
    }

    #[test]
    fn test_parse_file_with_thousand_separator() {
        let body = get_stock_page("test/data/stocks/amunid_thousand_period.html").unwrap();
        let db = parse_stock_price(&body).unwrap();

        assert_eq!(db.name, "AMUNDI ETF LEVERAGED MSCI USA DAILY UCITS ETF - EUR ACC");
        assert_eq!(db.wkn, "A0X8ZS");
        assert_eq!(db.date, "27.10.17");
        assert_eq!(db.price, 1396.47f32);
        assert_eq!(db.currency, "EUR");
    }

    #[test]
    fn test_parse_file_no_exact_match() {
        let body = get_stock_page("test/data/stocks/no_exact_match.html").unwrap();
        let db = parse_stock_price(&body);

        assert!(db.is_err());
    }

    #[test]
    fn test_parse_online_ok() {
        ::utils::console::init(::config::Verbosity::QUIET);
        let db = scrape_stock_price("Deutsche Bank", None::<fn(_)>).unwrap();

        assert_eq!(db.name, "DEUTSCHE BANK");
        assert_eq!(db.wkn, "514000");
        assert!(db.price > 0.00f32);
        assert_eq!(db.currency, "EUR");
    }

    #[test]
    fn test_parse_online_with_thousand_separator() {
        ::utils::console::init(::config::Verbosity::QUIET);
        let db = scrape_stock_price("A0X8ZS", None::<fn(_)>).unwrap();

        assert_eq!(db.name, "AMUNDI ETF LEVERAGED MSCI USA DAILY UCITS ETF - EUR ACC");
        assert_eq!(db.wkn, "A0X8ZS");
        assert!(db.price > 0.00_f32);
        assert_eq!(db.currency, "EUR");
    }

    #[test]
    fn test_parse_online_no_exact_match() {
        ::utils::console::init(::config::Verbosity::QUIET);
        let result = scrape_stock_price("Deutsche", None::<fn(_)>);

        // TODO: This needs to be nicer.
        let result_is_not_unique = match result.unwrap_err() {
            Error(ErrorKind::ComdirectSearchResultNotUnique, _) => true,
            _ => false
        };
        assert!(result_is_not_unique);
    }
}
