pub use self::search::search_documents;
pub use self::upload::upload_document;

mod search {
    use utils::console::*;

    use hyper::Client;
    use hyper::header::{ContentType, Authorization, Bearer, Accept, qitem};
    use hyper::net::HttpsConnector;
    use hyper_native_tls::NativeTlsClient;
    use serde_json;
    use std::io::Read;
    use std::str;

    error_chain! {
        errors {
            HttpSearchCallFailed {
                description("failed to make http search call")
                display("failed to make http search call")
            }
        }
    }


    #[derive(Serialize, Debug)]
    struct Search<'a> {
        action: &'a str,
        params: Params<'a>,
    }

    #[derive(Serialize, Debug)]
    struct Params<'a> {
        query: Query<'a>,
        filter: Filter<'a>,
    }

    #[derive(Serialize, Debug)]
    struct Query<'a> {
        #[serde(skip_serializing_if = "Option::is_none")] text: Option<&'a str>,
    }

    #[derive(Serialize, Debug)]
    struct Filter<'a> {
        #[serde(skip_serializing_if = "Option::is_none")] filenames: Option<Vec<&'a str>>,
        #[serde(skip_serializing_if = "Option::is_none")] tags: Option<Vec<&'a str>>,
    }

    impl<'a> Search<'a> {
        pub fn new(filenames: Option<Vec<&'a str>> , tags: Option<Vec<&'a str>>, fulltext: Option<&'a str>) -> Self {
            let filter = Filter { filenames: filenames, tags: tags };
            let query = Query { text: fulltext };
            let params = Params { query: query, filter: filter };
            let search = Search { action: "search", params: params };

            search
        }
    }

    pub fn search_documents(
        access_token: &str,
        filenames: Option<Vec<&str>>,
        tags: Option<Vec<&str>>,
        fulltext: Option<&str>) -> Result<String> {

        do_search_documents(access_token, filenames, tags, fulltext)
            .chain_err(|| ErrorKind::HttpSearchCallFailed)
    }

    fn do_search_documents(
        access_token: &str,
        filenames: Option<Vec<&str>>,
        tags: Option<Vec<&str>>,
        fulltext: Option<&str>) -> Result<String> {

        let search = Search::new(filenames, tags, fulltext);
        let search_json = serde_json::to_string(&search).chain_err(|| "JSON serialization failed")?;

        verbose(format!("search = '{:?}'", search_json));

        let ssl = NativeTlsClient::new().chain_err(|| "Failed to create TLS client")?;
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        let request = client.post("https://api.centerdevice.de/v2/documents")
            .header(Authorization(Bearer { token: access_token.to_string() }))
            .header(ContentType(mime!(Application/Json)))
            .header(Accept(vec![qitem(mime!(Application/Json; Charset=Utf8))]))
            .body(&search_json);

        let mut response = request.send().chain_err(|| "Failed to finish http request")?;

        let mut body = Vec::new();
        response.read_to_end(&mut body).chain_err(|| "Failed to read server response")?;
        let response_body = String::from_utf8_lossy(&body).to_string();

        Ok(response_body)
    }
}

mod upload {
    use utils::console::*;

    use crypto::digest::Digest;
    use crypto::sha2::Sha256;
    use hyper::client::Request;
    use hyper::method::Method;
    use hyper::header::{Headers, ContentDisposition, DispositionParam, DispositionType, ContentType,
                        Authorization, Bearer, Accept, qitem};
    use hyper::net::HttpsConnector;
    use hyper_native_tls::NativeTlsClient;
    use mime::Mime;
    use mime_multipart::{Node, Part, FilePart, write_multipart};
    use serde_json;
    use std::fs;
    use std::io::Read;
    use std::path::Path;
    use std::str;

    error_chain! {
        errors {
            HttpUploadCallFailed {
                description("failed to make http upload call")
                display("failed to make http upload call")
            }
        }
    }

    #[derive(Serialize, Debug)]
    struct DocumentMetadata<'a> {
        metadata: Metadata<'a>,
    }

    #[derive(Serialize, Debug)]
    struct Metadata<'a> {
        document: Document<'a>,
        #[serde(skip_serializing_if = "Option::is_none")] actions: Option<Actions<'a>>,
    }

    #[derive(Serialize, Debug)]
    struct Document<'a> {
        filename: &'a str,
        size: u64,
        #[serde(skip_serializing_if = "Option::is_none")] title: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")] author: Option<&'a str>,
    }

    #[derive(Serialize, Debug)]
    struct Actions<'a> {
        #[serde(skip_serializing_if = "Option::is_none", rename(serialize = "add-tag"))] tags: Option<Vec<&'a str>>
    }

    impl<'a> DocumentMetadata<'a> {
        pub fn new(filename: &'a str, size: u64, title: Option<&'a str>, tags: Option<Vec<&'a str>>) -> Self {
            let document = Document { filename: filename, size: size, title: title, author: None };
            let actions = Actions { tags: tags };
            let metadata = Metadata { document: document, actions: Some(actions) };
            DocumentMetadata { metadata: metadata }
        }
    }

    pub fn upload_document(
        access_token: &str,
        file: &Path,
        filename: &str,
        mime: Mime,
        title: Option<&str>,
        tags: Option<Vec<&str>>)
        -> Result<String> {
        do_upload_document(access_token, file, filename, mime, title, tags)
            .chain_err(|| ErrorKind::HttpUploadCallFailed)
    }

    fn do_upload_document(
        access_token: &str,
        file: &Path,
        filename: &str,
        mime: Mime,
        title: Option<&str>,
        tags: Option<Vec<&str>>) -> Result<String> {
        let doc_metadata_json_bytes = create_doc_metadata(file, filename, title, tags)?.into_bytes();
        let boundary = generate_boundary(&doc_metadata_json_bytes);
        let boundary_bytes = boundary.clone().into_bytes();
        let nodes = create_multipart_nodes(
            doc_metadata_json_bytes, filename.to_string(), file, mime).chain_err(|| "Failed to create form-data")?;

        let ssl = NativeTlsClient::new().chain_err(|| "Failed to create TLS client")?;
        let connector = HttpsConnector::new(ssl);
        let url = ::hyper::Url::parse("https://api.centerdevice.de/v2/documents").chain_err(|| "Failed to parse URL")?;
        let mut client = Request::with_connector(Method::Post, url, &connector).chain_err(|| "Failed to create client")?;
        client.headers_mut().set(Authorization(Bearer { token: access_token.to_string() }));
        client.headers_mut().set(ContentType(mime!(Multipart/FormData; Boundary=(boundary))));
        client.headers_mut().set(Accept(vec![qitem(mime!(Application/Json; Charset=Utf8))]));

        let mut request = client.start().chain_err(|| "Failed to start HTTP connection")?;
        write_multipart(&mut request, &boundary_bytes, &nodes).chain_err(|| "Failed to send multipart form-data")?;
        let mut response = request.send().chain_err(|| "Failed to finish http request")?;

        let mut body = Vec::new();
        response.read_to_end(&mut body).chain_err(|| "Failed to read server response")?;
        let response_body = String::from_utf8_lossy(&body).to_string();

        Ok(response_body)
    }

    fn create_doc_metadata(file: &Path, filename: &str, title: Option<&str>, tags: Option<Vec<&str>>) -> Result<String> {
        let size = fs::metadata(file).chain_err(|| "Failed to get metadata for file")?.len();
        let doc_metadata = DocumentMetadata::new(filename, size, title, tags);
        let doc_metadata_json = serde_json::to_string(&doc_metadata).chain_err(|| "JSON serialization failed")?;

        verbose(format!("document metadata = '{:?}'", doc_metadata_json));

        Ok(doc_metadata_json)
    }

    fn create_multipart_nodes(json_bytes: Vec<u8>, filename: String, file: &Path, mime: Mime) -> Result<Vec<Node>> {
        let mut nodes: Vec<Node> = Vec::with_capacity(2);

        let mut h = Headers::new();
        h.set(ContentType(mime!(Application/Json)));
        h.set(ContentDisposition {
            disposition: DispositionType::Ext("form-data".to_string()),
            parameters: vec![DispositionParam::Ext("name".to_string(), "metadata".to_string())],
        });
        nodes.push(Node::Part(Part {
            headers: h,
            body: json_bytes
        }));

        let mut h = Headers::new();
        h.set(ContentType(mime));
        h.set(ContentDisposition {
            disposition: DispositionType::Ext("form-data".to_string()),
            parameters: vec![DispositionParam::Ext("name".to_string(), "document".to_string()),
                             DispositionParam::Ext("filename".to_string(), filename)],
        });
        nodes.push(Node::File(FilePart::new(h, file)));

        Ok(nodes)
    }

    // CenterDevice / Jersey does not accept special characters in boundary; thus, we build it ourselves.
    fn generate_boundary(seed: &[u8]) -> String {
        let mut sha = Sha256::new();
        sha.input(seed);
        let output = format!("Boundary_{}", sha.result_str());

        output
    }
}