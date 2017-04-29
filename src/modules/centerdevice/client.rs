pub use self::collections::search_collections;
pub use self::delete::delete_documents;
pub use self::download::download_document;
pub use self::refresh_token::refresh_token;
pub use self::search::search_documents;
pub use self::upload::upload_document;

use hyper::Client;
use hyper::method::Method;
use hyper::client::RequestBuilder;
use hyper::header::{Authorization, Bearer};

error_chain! {}

fn prepare_request<'a, 'b>(client: &'a Client, method: Method, url: &'b str, token: String) -> Result<RequestBuilder<'a>> {
    let request = match method {
        Method::Get => client.get(url),
        Method::Post => client.post(url),
        _ => unimplemented!(),
    }.header(Authorization(Bearer { token: token }));

    Ok(request)
}

pub mod collections {
    use super::prepare_request;
    use net::http::tls_client;

    use utils::console::*;

    use hyper::method::Method;
    use serde_json;
    use serde_urlencoded;
    use std::io::Read;
    use std::str;

    error_chain! {
        errors {
            HttpCollectionCallFailed {
                description("failed to make http collection call")
                display("failed to make http collection call")
            }
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct CollectionsResult {
        pub collections: Vec<Collection>,
    }

    impl CollectionsResult {
        pub fn filter(self, regex: &str) -> CollectionsResult {
            let regex = regex;
            let collections = self.collections.into_iter().filter(|c| c.name.contains(regex)).collect();
            CollectionsResult { collections: collections }
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Collection {
        pub id: String,
        pub public: bool,
        pub name: String,
    }

    pub fn search_collections(
        access_token: &str,
        name: Option<&str>,
        include_public: bool,
        filter: Option<&str>
    ) -> Result<String> {
        let json = do_search_collections(access_token, name, include_public)
            .chain_err(|| ErrorKind::HttpCollectionCallFailed);
        if filter.is_none() {
            return json;
        }

        let collections_result: CollectionsResult = serde_json::from_str(&json.unwrap())
            .chain_err(|| "Failed to parse JSON")?;
        let collections_result = collections_result.filter(filter.unwrap());

        let json = serde_json::to_string(&collections_result).chain_err(|| "Failed to serialize JSON")?;
        Ok(json)
    }

    fn do_search_collections(
        access_token: &str,
        name: Option<&str>,
        include_public: bool
    ) -> Result<String> {
        let mut parameters = vec![
            ("include-public", format!("{}", include_public)),
        ];
        if name.is_some() {
            parameters.push(("name", name.unwrap().to_string()))
        }
        let parameters_enc = serde_urlencoded::to_string(&parameters).chain_err(|| "URL serialization failed")?;
        let url = format!("https://api.centerdevice.de/v2/collections?{}", parameters_enc);

        verboseln(format!("collections search = {}", url));

        let client = tls_client().chain_err(|| "Failed to create HTTP client")?;
        let request = prepare_request(&client, Method::Get, &url, access_token.to_string())
            .chain_err(|| "Failed to create CenterDevice client")?;

        let mut response = request.send().chain_err(|| "Failed to finish http request")?;

        let mut body = Vec::new();
        response.read_to_end(&mut body).chain_err(|| "Failed to read server response")?;
        let response_body = String::from_utf8_lossy(&body).to_string();

        Ok(response_body)
    }
}

mod delete {
    use super::prepare_request;
    use net::http::tls_client;

    use hyper::method::Method;
    use hyper::header::{ContentType, Accept, qitem};
    use serde_json;
    use std::io::Read;
    use std::str;

    error_chain! {
        errors {
            HttpUploadCallFailed {
                description("failed to make http delete call")
                display("failed to make http delete call")
            }
        }
    }

    #[derive(Serialize, Debug)]
    struct DeleteAction<'a> {
        action: &'a str,
        params: Documents<'a>,
    }

    #[derive(Serialize, Debug)]
    struct Documents<'a> {
        documents: Vec<&'a str>,
    }

    impl<'a> DeleteAction<'a> {
        pub fn new(documents: Vec<&'a str>) -> Self {
            let params = Documents { documents: documents };
            DeleteAction { action: "delete", params: params }
        }
    }

    pub fn delete_documents(access_token: &str, document_ids: Vec<&str>) -> Result<String> {
        do_delete_documents(access_token, document_ids).chain_err(|| ErrorKind::HttpUploadCallFailed)
    }

    fn do_delete_documents(access_token: &str, document_ids: Vec<&str>) -> Result<String> {
        let delete = DeleteAction::new(document_ids);
        let delete_json = serde_json::to_string(&delete).chain_err(|| "JSON serialization failed")?;

        let url = "https://api.centerdevice.de/v2/documents";
        let client = tls_client().chain_err(|| "Failed to create HTTP client")?;
        let request = prepare_request(&client, Method::Post, url, access_token.to_string())
            .chain_err(|| "Failed to create CenterDevice client")?
            .header(ContentType(mime!(Application / Json)))
            .header(Accept(vec![qitem(mime!(Application/ Json; Charset = Utf8))]))
            .body(&delete_json);

        let mut response = request.send().chain_err(|| "Failed to finish http request")?;

        let mut body = Vec::new();
        response.read_to_end(&mut body).chain_err(|| "Failed to read server response")?;
        let response_body = String::from_utf8_lossy(&body).to_string();

        Ok(response_body)
    }
}

mod download {
    use super::prepare_request;
    use net::http::tls_client;
    use utils::console::*;

    use hyper::client::Response;
    use hyper::header::{ContentType, ContentDisposition, ContentLength, DispositionParam, Authorization, Bearer};
    use hyper::method::Method;
    use std::fs::File;
    use std::io;
    use std::io::{Read, Write};
    use std::str;

    error_chain! {
        errors {
            HttpDownloadCallFailed {
                description("failed to make http download call")
                display("failed to make http download call")
            }

            FailedSetFilename {
                description("failed set filename from cli parameter and content disposition header")
                display("failed set filename from cli parameter and content disposition header")
            }

            FailedContentLength {
                description("failed get content length from response header")
                display("failed get content length from response header")
            }
        }
    }


    pub fn download_document(
        access_token: &str,
        filename: Option<&str>,
        document_id: &str) -> Result<()> {
        do_download_document(access_token, filename, document_id)
            .chain_err(|| ErrorKind::HttpDownloadCallFailed)
    }

    fn do_download_document(
        access_token: &str,
        filename: Option<&str>,
        document_id: &str
    ) -> Result<()> {
        let url = format!("https://api.centerdevice.de/v2/document/{}", document_id);
        let client = tls_client().chain_err(|| "Failed to create HTTP client")?;
        let request = prepare_request(&client, Method::Get, &url, access_token.to_string())
            .chain_err(|| "Failed to create CenterDevice client")?
            .header(Authorization(Bearer { token: access_token.to_string() }))
            .header(ContentType(mime!(Star/Star)));

        let mut response = request.send().chain_err(|| "Failed to finish http request")?;

        let filename = get_filename(filename, &response)?;
        let size = get_content_length(&response)? as usize;

        verbose(format!("Retrieving {} bytes ", size));
        let w_size = write_to_file(&filename, &mut response)?;
        verboseln("done.");

        if w_size != size {
            bail!("Retrieved number of bytes ({}) does not match expected bytes ({}).", w_size, size);
        }

        Ok(())
    }

    fn get_filename(filename: Option<&str>, response: &Response) -> Result<String> {
        let filename = if let Some(_filename) = filename {
            _filename
        } else {
            let mut _filename = None;
            let content_disposition: &ContentDisposition = response.headers.get()
                .ok_or(ErrorKind::FailedSetFilename)?;
            for cp in &content_disposition.parameters {
                if let DispositionParam::Filename(_, _, ref f) = *cp {
                    _filename = Some(
                        str::from_utf8(f).chain_err(|| ErrorKind::FailedSetFilename)?);
                    break;
                }
            }
            _filename.ok_or(ErrorKind::FailedSetFilename)?
        };

        Ok(filename.to_string())
    }

    fn get_content_length(response: &Response) -> Result<u64> {
        let content_length: &ContentLength = response.headers.get()
            .ok_or(ErrorKind::FailedContentLength)?;
        let &ContentLength(size) = content_length;
        Ok(size)
    }

    fn write_to_file(filename: &str, response: &mut Response) -> Result<usize> {
        let mut out = File::create(filename).chain_err(|| "Failed to create destination file")?;

        let mut buf = [0; 128 * 1024];
        let mut written: usize = 0;
        let mut mega_bytes: usize = 0;
        loop {
            let len = match response.read(&mut buf) {
                Ok(0) => break,  // EOF.
                Ok(len) => len,
                Err(ref err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => bail!("Could not read from server response")
            };
            written += len;
            if (written / (1024 * 1024)) - mega_bytes > 0 {
                mega_bytes += 1;
                verbose(".");
            }
            out.write_all(&buf[..len]).chain_err(|| "Could not write to file")?;
        }

        Ok(written)
    }
}

mod refresh_token {
    use net::http::tls_client;

    use hyper::header::{ContentType, Authorization, Basic};
    use std::io::Read;
    use std::str;

    error_chain! {
        errors {
            HttpUploadCallFailed {
                description("failed to make http refresh access token call")
                display("failed to make http refresh access token call")
            }
        }
    }

    pub fn refresh_token(refresh_token: &str, client_id: &str, client_secret: &str) -> Result<String> {
        do_refresh_token(refresh_token, client_id, client_secret).chain_err(|| ErrorKind::HttpUploadCallFailed)
    }

    fn do_refresh_token(refresh_token: &str, client_id: &str, client_secret: &str) -> Result<String> {
        let body = format!("grant_type=refresh_token&refresh_token={}", refresh_token);
        let url = "https://auth.centerdevice.de/token";
        let client = tls_client().chain_err(|| "Could not create TLS client")?;
        let mut response = client
            .post(url)
            .header(Authorization(Basic { username: client_id.to_string(), password: Some(client_secret.to_string()) }))
            .header(ContentType(mime!(Application / WwwFormUrlEncoded)))
            .body(&body)
            .send()
            .chain_err(|| "Failed to finish HTTP request")?;

        let mut body = Vec::new();
        response.read_to_end(&mut body).chain_err(|| "Failed to read HTTP response")?;
        let response_body = String::from_utf8_lossy(&body).to_string();

        Ok(response_body)
    }
}

pub mod search {
    use super::prepare_request;
    use net::http::tls_client;

    use utils::console::*;

    use hyper::header::{ContentType, Accept, qitem};
    use hyper::method::Method;
    use serde::Serialize;
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

    #[derive(PartialEq, Debug)]
    pub enum NamedSearches {
        None,
        PublicCollections
    }

    #[derive(Serialize, Debug)]
    struct Search<'a, T: Serialize> {
        action: &'a str,
        params: Params<'a, T>,
    }

    #[derive(Serialize, Debug)]
    struct Params<'a, T: Serialize> {
        query: Query<'a>,
        filter: Filter<'a>,
        #[serde(skip_serializing_if = "Option::is_none")] named: Option<Vec<Named<'a, T>>>,
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

    #[derive(Serialize, Debug)]
    struct Named<'a, T: Serialize> {
        name: &'a str,
        params: T,
    }

    #[derive(Serialize, Debug)]
    struct Include {
        include: bool
    }

    impl<'a, T: Serialize> Search<'a, T> {
        pub fn new(
            filenames: Option<Vec<&'a str>>,
            tags: Option<Vec<&'a str>>,
            fulltext: Option<&'a str>,
            named: Option<Vec<Named<'a, T>>>) -> Self {
            let filter = Filter { filenames: filenames, tags: tags };
            let query = Query { text: fulltext };
            let params = Params { query: query, filter: filter, named: named };

            Search { action: "search", params: params }
        }
    }

    pub fn search_documents(
        access_token: &str,
        filenames: Option<Vec<&str>>,
        tags: Option<Vec<&str>>,
        fulltext: Option<&str>,
        named_searches: NamedSearches) -> Result<String> {
        do_search_documents(access_token, filenames, tags, fulltext, named_searches)
            .chain_err(|| ErrorKind::HttpSearchCallFailed)
    }

    fn do_search_documents(
        access_token: &str,
        filenames: Option<Vec<&str>>,
        tags: Option<Vec<&str>>,
        fulltext: Option<&str>,
        named_searches: NamedSearches) -> Result<String> {
        let named: Option<Vec<Named<Include>>> = match named_searches {
            NamedSearches::None => None,
            NamedSearches::PublicCollections => {
                let includes = Include { include: true };
                let named = vec![Named { name: "public-collections", params: includes }];
                Some(named)
            }
        };
        let search = Search::new(filenames, tags, fulltext, named);
        let search_json = serde_json::to_string(&search).chain_err(|| "JSON serialization failed")?;

        verboseln(format!("search = '{:?}'", search_json));

        let url = "https://api.centerdevice.de/v2/documents";
        let client = tls_client().chain_err(|| "Failed to create HTTP client")?;
        let request = prepare_request(&client, Method::Post, url, access_token.to_string())
            .chain_err(|| "Failed to create CenterDevice client")?
            .header(ContentType(mime!(Application / Json)))
            .header(Accept(vec![qitem(mime!(Application/ Json; Charset = Utf8))]))
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
        #[serde(skip_serializing_if = "Option::is_none", rename(serialize = "add-tag"))] tags: Option<Vec<&'a str>>,
        #[serde(skip_serializing_if = "Option::is_none", rename(serialize = "add-to-collection"))] collections: Option<Vec<&'a str>>,
    }

    impl<'a> DocumentMetadata<'a> {
        pub fn new(
            filename: &'a str,
            size: u64,
            title: Option<&'a str>,
            tags: Option<Vec<&'a str>>,
            collections: Option<Vec<&'a str>>
        ) -> Self {
            let document = Document { filename: filename, size: size, title: title, author: None };
            let actions = Actions { tags: tags, collections: collections };
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
        tags: Option<Vec<&str>>,
        collections: Option<Vec<&str>>
    ) -> Result<String> {
        do_upload_document(access_token, file, filename, mime, title, tags, collections)
            .chain_err(|| ErrorKind::HttpUploadCallFailed)
    }

    fn do_upload_document(
        access_token: &str,
        file: &Path,
        filename: &str,
        mime: Mime,
        title: Option<&str>,
        tags: Option<Vec<&str>>,
        collections: Option<Vec<&str>>
    ) -> Result<String> {
        let doc_metadata_json_bytes = create_doc_metadata(file, filename, title, tags, collections)?.into_bytes();
        let boundary = generate_boundary(&doc_metadata_json_bytes);
        let boundary_bytes = boundary.clone().into_bytes();
        let nodes = create_multipart_nodes(
            doc_metadata_json_bytes, filename.to_string(), file, mime).chain_err(|| "Failed to create form-data")?;

        let ssl = NativeTlsClient::new().chain_err(|| "Failed to create TLS client")?;
        let connector = HttpsConnector::new(ssl);
        let url = ::hyper::Url::parse("https://api.centerdevice.de/v2/documents").chain_err(|| "Failed to parse URL")?;
        let mut client = Request::with_connector(Method::Post, url, &connector).chain_err(|| "Failed to create client")?;
        client.headers_mut().set(Authorization(Bearer { token: access_token.to_string() }));
        client.headers_mut().set(ContentType(mime!(Multipart / FormData; Boundary = (boundary))));
        client.headers_mut().set(Accept(vec![qitem(mime!(Application / Json; Charset = Utf8))]));

        let mut request = client.start().chain_err(|| "Failed to start HTTP connection")?;
        write_multipart(&mut request, &boundary_bytes, &nodes).chain_err(|| "Failed to send multipart form-data")?;
        let mut response = request.send().chain_err(|| "Failed to finish http request")?;

        let mut body = Vec::new();
        response.read_to_end(&mut body).chain_err(|| "Failed to read server response")?;
        let response_body = String::from_utf8_lossy(&body).to_string();

        Ok(response_body)
    }

    fn create_doc_metadata(
        file: &Path,
        filename: &str,
        title: Option<&str>,
        tags: Option<Vec<&str>>,
        collections: Option<Vec<&str>>
    ) -> Result<String> {
        let size = fs::metadata(file).chain_err(|| "Failed to get metadata for file")?.len();
        let doc_metadata = DocumentMetadata::new(filename, size, title, tags, collections);
        let doc_metadata_json = serde_json::to_string(&doc_metadata).chain_err(|| "JSON serialization failed")?;

        verboseln(format!("document metadata = '{:?}'", doc_metadata_json));

        Ok(doc_metadata_json)
    }

    fn create_multipart_nodes(json_bytes: Vec<u8>, filename: String, file: &Path, mime: Mime) -> Result<Vec<Node>> {
        let mut nodes: Vec<Node> = Vec::with_capacity(2);

        let mut h = Headers::new();
        h.set(ContentType(mime!(Application / Json)));
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


