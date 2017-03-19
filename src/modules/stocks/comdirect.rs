use super::StockPrice;
use utils::console::*;

use hyper::Client;
use hyper::header::Connection;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};
use serde_urlencoded;
use std::io::Read;

static BASE_URL: &'static str = "https://www.comdirect.de/inf/search/all.html";

error_chain! {
    errors {
        ComdirectScrapingFailed(info: String) {
            description("Did not find information in HTML document")
            display("Did not find {} in HTML document", info)
        }
    }
}

pub fn scrape_stock_price(search: String) -> Result<StockPrice> {
    let parameters = &[("SEARCH_VALUE", search) ];
    let parameters_enc = serde_urlencoded::to_string(&parameters)
        .chain_err(|| "Could not encode URL parameters")?;
    let url = format!("{}?{}", BASE_URL, parameters_enc);

    let body = get_stock_page(&url).unwrap();
    let stock_price = parse_stock_price(&body)?;

    Ok(stock_price)
}

fn get_stock_page(url: &str) -> Result<Vec<u8>> {
    let ssl = NativeTlsClient::new().chain_err(|| "Could not create TLS client")?;
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    info("Sending search request ...");
    let mut response = client.get(url).header(Connection::close()).send()
        .chain_err(|| "Could not send request")?;

    let mut body = Vec::new();
    let size = response.read_to_end(&mut body)
        .chain_err(|| "Could not read response body")?;
    info(format!("Received {} bytes body.", size));

    Ok(body)
}

fn parse_stock_price(body: &[u8]) -> Result<StockPrice> {
    let document = Document::from_read(body).chain_err(|| "Could not parse HTML in response body")?;

    let no_exact_match = document.find(Class("Informer")).nth(0).is_some();
    if no_exact_match {
        bail!("Could not find exact match for your search");
    }

    let name = document.find(Name("h1")).nth(0)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("name".to_string()))?.text();
    let price_currency = document.find(Class("price")).nth(0)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock price".to_string()))?.text();
    let date = document.find(Attr("id", "keyelement_kurs_update").descendant(Class("date"))).last()
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock price date".to_string()))?.text();
    let wkn = document.find(Class("clearfix").descendant(Name("table")).descendant(Name("tr")).descendant(Class("right"))).last()
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("stock WKN".to_string()))?.text();

    let mut pc_split = price_currency.split_whitespace();
    let price = pc_split.nth(0)
        .ok_or_else(|| ErrorKind::ComdirectScrapingFailed("price".to_string()))?
        .trim()
        .replace(",", ".") // decimal separation
        .parse()
        .chain_err(|| ErrorKind::ComdirectScrapingFailed("price".to_string()))?;
    let currency = pc_split.nth(0).ok_or_else(|| ErrorKind::ComdirectScrapingFailed("currency".to_string()))?;

    let stock_price = StockPrice {
        name: name.trim().to_string(),
        wkn: wkn.trim().to_string(),
        date: date.to_string(),
        price: price,
        currency: currency.trim().to_string(),
    };

    Ok(stock_price)
}


#[cfg(test)]
mod test {
    use super::*;
    use std::io::Read;
    use std::fs::File;

    fn get_stock_page(file: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut f = File::open(file)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    #[test]
    fn test_parse_ok() {
        let body = get_stock_page("test/data/stocks/deutsche_bank.html").unwrap();
        let db = parse_stock_price(&body).unwrap();

        assert_eq!(db.name, "Deutsche Bank AG Namens-Aktien o.N.");
        assert_eq!(db.wkn, "514000");
        assert_eq!(db.date, "10.03.17\u{a0}\u{a0}17:35:07\u{a0}Uhr");
        assert_eq!(db.price, "18,26 EUR");
    }

    #[test]
    fn test_parse_no_exact_match() {
        let body = get_stock_page("test/data/stocks/no_exact_match.html").unwrap();
        let db = parse_stock_price(&body);

        assert!(db.is_err());
    }
}
