pub use self::download::download_document;
pub use self::search::search_documents;
pub use self::upload::upload_document;

mod download {
    use utils::console::*;

    use hyper::Client;
    use hyper::client::Response;
    use hyper::header::{ContentType, ContentDisposition, ContentLength, DispositionParam, Authorization, Bearer};
    use hyper::net::HttpsConnector;
    use hyper_native_tls::NativeTlsClient;
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
        document_id: &str) -> Result<()> {
        let ssl = NativeTlsClient::new().chain_err(|| "Failed to create TLS client")?;
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        let url = format!("https://api.centerdevice.de/v2/document/{}", document_id);
        let request = client.get(&url)
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

mod search {
    use utils::console::*;

    use hyper::Client;
    use hyper::header::{ContentType, Authorization, Bearer, Accept, qitem};
    use hyper::net::HttpsConnector;
    use hyper_native_tls::NativeTlsClient;
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


    #[derive(Serialize, Debug)]
    struct Search<'a> {
        action: &'a str,
        params: Params<'a>,
    }

    #[derive(Serialize, Debug)]
    struct Params<'a> {
        query: Query<'a>,
        filter: Filter<'a>,
        #[serde(skip_serializing_if = "Vec::is_empty")] named: Vec<Named<'a>>,
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
    struct Named<'a> {
        name: &'a str,
        params: Include,
    }

    #[derive(Serialize, Debug)]
    struct Include {
        include: bool
    }

    impl<'a> Search<'a> {
        pub fn new(filenames: Option<Vec<&'a str>>, tags: Option<Vec<&'a str>>, fulltext: Option<&'a str>) -> Self {
            let filter = Filter { filenames: filenames, tags: tags };
            let query = Query { text: fulltext };
            let includes = Include{ include: true };
            let named = vec![Named { name: "public-collections", params: includes }];
            let params = Params { query: query, filter: filter, named: named};

            Search { action: "search", params: params }
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

        verboseln(format!("search = '{:?}'", search_json));

        let ssl = NativeTlsClient::new().chain_err(|| "Failed to create TLS client")?;
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        let request = client.post("https://api.centerdevice.de/v2/documents")
            .header(Authorization(Bearer { token: access_token.to_string() }))
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

    fn create_doc_metadata(file: &Path, filename: &str, title: Option<&str>, tags: Option<Vec<&str>>) -> Result<String> {
        let size = fs::metadata(file).chain_err(|| "Failed to get metadata for file")?.len();
        let doc_metadata = DocumentMetadata::new(filename, size, title, tags);
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