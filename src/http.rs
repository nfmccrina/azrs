use crate::http::crypto::RequestSigner;
use async_trait::async_trait;
use std::error::Error;
use surf::Response;

pub mod crypto;

#[async_trait]
pub trait AzRsHttpClient {
    async fn get(
        &self,
        path: &str,
        query_params: Option<Vec<(String, String)>>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<Response, Box<dyn Error>>;
}

pub struct AzRsHttpClientSurf {
    host: String,
    api_version: String,
    credential: String,
    secret: String,
}

#[async_trait]
impl AzRsHttpClient for AzRsHttpClientSurf {
    async fn get(
        &self,
        path: &str,
        query_params: Option<Vec<(String, String)>>,
        headers: Option<Vec<(String, String)>>,
    ) -> Result<Response, Box<dyn Error>> {
        let client = surf::Client::new();
        let query_string = QueryStringBuilder::new(&self.api_version)
            .add_params(&mut match query_params {
                None => {
                    let v: Vec<(String, String)> = Vec::new();
                    v
                }
                Some(v) => v,
            })
            .build();
        let full_url = String::from("https://") + &self.host + path + &query_string;

        let req_signer = RequestSigner::new(&self.credential, &self.secret);

        let req = client.get(&full_url);
        let final_req;

        if headers != Option::None {
            final_req = headers
                .unwrap()
                .iter()
                .fold(req, |r, h| r.set_header((&h.0).parse().unwrap(), &h.1));
        } else {
            final_req = req;
        }

        Ok(final_req.middleware(req_signer).await?)
    }
}

impl AzRsHttpClientSurf {
    pub fn new(host: String, api_version: String, credential: String, secret: String) -> Self {
        AzRsHttpClientSurf {
            host,
            api_version,
            credential,
            secret,
        }
    }
}

struct UrlParser {}

impl UrlParser {
    fn get_path_and_query(url: &str, host: &str) -> String {
        match url
            .replace("http://", "https://")
            .replace("https://", "")
            .replace(host, "")
            .as_str()
        {
            "" => String::from("/"),
            s => String::from(s),
        }
    }
}

#[cfg(test)]
mod url_parser_tests {
    use super::*;

    #[test]
    fn it_should_return_slash_if_root() {
        assert_eq!(
            "/",
            UrlParser::get_path_and_query("https://www.example.com", "www.example.com")
        );
    }

    #[test]
    fn it_should_return_only_path_if_no_query() {
        assert_eq!(
            "/test/path",
            UrlParser::get_path_and_query("https://www.example.com/test/path", "www.example.com")
        );
    }

    #[test]
    fn it_should_handle_http_or_https() {
        assert_eq!(
            "/test/path",
            UrlParser::get_path_and_query("https://www.example.com/test/path", "www.example.com")
        );
        assert_eq!(
            UrlParser::get_path_and_query("http://www.example.com/test/path", "www.example.com"),
            UrlParser::get_path_and_query("https://www.example.com/test/path", "www.example.com")
        );
    }

    #[test]
    fn it_should_include_path_and_query_if_exist() {
        assert_eq!(
            "/test/path?foo=bar&api-version=1.0",
            UrlParser::get_path_and_query(
                "https://www.example.com/test/path?foo=bar&api-version=1.0",
                "www.example.com"
            )
        );
    }
}

#[derive(Debug)]
struct QueryStringBuilder {
    params: Vec<(String, String)>,
}

impl QueryStringBuilder {
    fn new(api_version: &str) -> Self {
        QueryStringBuilder {
            params: vec![(String::from("api-version"), String::from(api_version))],
        }
    }

    #[allow(dead_code)]
    fn add_param(&mut self, key: &str, value: &str) -> &QueryStringBuilder {
        let api_version = self.params.pop().unwrap();

        self.params
            .append(&mut vec![(String::from(key), String::from(value))]);

        self.params.push(api_version);
        self
    }

    fn add_params(&mut self, p: &mut Vec<(String, String)>) -> &QueryStringBuilder {
        let api_version = self.params.pop().unwrap();
        self.params.append(p);
        self.params.push(api_version);
        self
    }

    fn build(&self) -> String {
        String::from("?")
            + &self
                .params
                .iter()
                .map(|(k, v)| vec![String::from(k), String::from(v)].join("="))
                .collect::<Vec<String>>()
                .join("&")
    }
}

#[cfg(test)]
mod query_string_builder_tests {
    use super::*;

    #[test]
    fn it_contains_api_version_by_default() {
        let builder = QueryStringBuilder::new("1.0");

        assert_eq!("?api-version=1.0", builder.build());
    }

    #[test]
    fn it_can_add_custom_params() {
        let mut builder = QueryStringBuilder::new("1.0");

        assert_eq!(
            "?foo=bar&api-version=1.0",
            builder.add_param("foo", "bar").build()
        );
    }

    #[test]
    fn it_can_add_multiple_custom_params() {
        let mut builder = QueryStringBuilder::new("1.0");

        assert_eq!(
            "?foo=bar&foo2=bar2&api-version=1.0",
            builder
                .add_params(&mut vec![
                    (String::from("foo"), String::from("bar")),
                    (String::from("foo2"), String::from("bar2"))
                ])
                .build()
        );
    }
}
