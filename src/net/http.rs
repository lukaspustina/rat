use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

error_chain!{
    errors {
        FailedToCreateTlsClient {
            description("Failed to create TLS client")
            display("Failed to create TLS client")
        }
    }
}

pub fn tls_client() -> Result<Client> {
    let ssl = NativeTlsClient::new().chain_err(|| ErrorKind::FailedToCreateTlsClient)?;
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    Ok(client)
}
