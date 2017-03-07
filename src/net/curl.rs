use curl::easy::{Easy, List};
use itertools::Itertools;
use std::io::Read;
use std::str;

error_chain! {
    errors {
       CurlExecutionFailed {
            description("failed to execute curl function")
            display("failed to execute curl function")
        }

        ResponseCodeFailed {
            description("failed to get response code")
            display("failed to get response code")
        }
    }
}

pub enum HttpVerb {
    GET,
    POST,
}

pub fn curl(
    url: &str,
    verb: HttpVerb,
    headers: Option<&[&str]>,
    input: Option<&[u8]>,
    output: Option<&mut Vec<u8>>) -> Result<(u32)> {

    let mut easy = Easy::new();
    let mut list = List::new();

    // debug outout
    // easy.verbose(true)?;

    easy.url(url).chain_err(|| ErrorKind::CurlExecutionFailed)?;
    match verb {
        HttpVerb::GET  => easy.get(true).chain_err(|| ErrorKind::CurlExecutionFailed)?,
        HttpVerb::POST => {
            if let Some(input) = input {
                easy.post_field_size(input.len() as u64)
                    .chain_err(|| ErrorKind::CurlExecutionFailed)?;
            }
            easy.post(true).chain_err(|| ErrorKind::CurlExecutionFailed)?
        },
    }

    if let Some(headers) = headers {
        headers.iter().foreach(|h| list.append(h).unwrap() );
    }
    easy.http_headers(list).chain_err(|| ErrorKind::CurlExecutionFailed)?;

    {
        let mut in_buf: &[u8];
        let out_buf: &mut Vec<u8>;
        let mut transfer = easy.transfer();

        // Do not use if let, because we need to use `in_buf` declared _before_ `transfer`.
        if input.is_some() {
            in_buf = input.unwrap();
            transfer.read_function(|buf| {
                let res = in_buf.read(buf).unwrap_or(0);
                Ok(res)
            }).chain_err(|| ErrorKind::CurlExecutionFailed)?;
        }

        // Do not use if let, because we need to use `out_buf` declared _before_ `transfer`.
        if output.is_some() {
            out_buf = output.unwrap();
            transfer.write_function(|data| {
                out_buf.extend_from_slice(data);
                Ok(data.len())
            }).chain_err(|| ErrorKind::CurlExecutionFailed)?;
        }

        transfer.perform().chain_err(|| ErrorKind::CurlExecutionFailed)?
    }

    let response_code = easy.response_code().chain_err(|| ErrorKind::ResponseCodeFailed)?;

    Ok(response_code)
}

pub fn curl_json(
    url: &str,
    verb: HttpVerb,
    headers: Option<&[&str]>,
    input: Option<&[u8]>,
    output: Option<&mut Vec<u8>>) -> Result<(u32)> {

    let mut json_headers: Vec<&str> = vec!["Accept: application/json", "Content-Type: application/json"];
    if let Some(headers) = headers {
        json_headers.extend(headers.iter());
    }

    curl(url, verb, Some(&json_headers), input, output)
}
