use curl::easy::{Easy, List};
use curl::Error as CurlError;
use itertools::Itertools;
use std::io::Read;
use std::str;

pub enum HttpVerb {
    GET,
    POST,
}

pub fn curl(
    url: &str,
    verb: HttpVerb,
    headers: Option<&[&str]>,
    input: Option<&[u8]>,
    output: Option<&mut Vec<u8>>) -> Result<(u32), CurlError> {

    let mut easy = Easy::new();
    let mut list = List::new();

    // debug outout
    // easy.verbose(true)?;

    easy.url(url)?;
    match verb {
        HttpVerb::GET  => easy.get(true)?,
        HttpVerb::POST => {
            if let Some(input) = input {
                easy.post_field_size(input.len() as u64)?;
            }
            easy.post(true)?
        },
    }

    if let Some(headers) = headers {
        headers.iter().foreach(|h| list.append(h).unwrap() );
    }
    easy.http_headers(list)?;

    {
        let mut in_buf: &[u8];
        let out_buf: &mut Vec<u8>;
        let mut transfer = easy.transfer();

        if input.is_some() {
            in_buf = input.unwrap();
            transfer.read_function(|buf| {
                let res = in_buf.read(buf).unwrap_or(0);
                Ok(res)
            })?;
        }

        if output.is_some() {
            out_buf = output.unwrap();
            transfer.write_function(|data| {
                out_buf.extend_from_slice(data);
                Ok(data.len())
            })?;
        }

        transfer.perform()?;
    }

    let response_code = easy.response_code()?;

    Ok(response_code)
}

pub fn curl_json(
    url: &str,
    verb: HttpVerb,
    headers: Option<&[&str]>,
    input: Option<&[u8]>,
    output: Option<&mut Vec<u8>>) -> Result<(u32), CurlError> {

    let mut json_headers: Vec<&str> = vec!["Accept: application/json", "Content-Type: application/json"];
    if let Some(headers) = headers {
        json_headers.extend(headers.iter());
    }

    curl(url, verb, Some(&json_headers), input, output)
}
