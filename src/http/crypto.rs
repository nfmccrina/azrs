use base64;
use chrono::prelude::*;
use futures::future::BoxFuture;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use surf::middleware::{HttpClient, Middleware, Next, Request, Response};
use url::Url;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct RequestSigner {
    credential: String,
    secret: String,
}

impl<C: HttpClient> Middleware<C> for RequestSigner {
    fn handle<'a>(
        &'a self,
        req: Request,
        client: C,
        next: Next<'a, C>,
    ) -> BoxFuture<'a, Result<Response, http_types::Error>> {
        Box::pin(async move {
            let (x_ms_date, x_ms_content_sha256, authorization) = RequestSigner::sign_request(
                &req.method().to_string(),
                req.url().host_str().unwrap(),
                req.url().as_str(),
                Option::None,
                &self.credential,
                &self.secret,
            );

            let mut new_request =
                Request::new(req.method(), Url::parse(req.url().as_str()).unwrap());

            for (name, values) in req.iter() {
                new_request.insert_header(name.as_str(), values[0].as_str())?;
            }

            new_request.insert_header("x-ms-date", x_ms_date)?;
            new_request.insert_header("x-ms-content-sha256", x_ms_content_sha256)?;
            new_request.insert_header("Authorization", authorization)?;

            Ok(next.run(new_request, client).await?)
        })
    }
}

impl RequestSigner {
    pub fn new(credential: &str, secret: &str) -> Self {
        RequestSigner {
            credential: String::from(credential),
            secret: String::from(secret),
        }
    }

    fn sign_request(
        method: &str,
        host: &str,
        url: &str,
        body: Option<&str>,
        credential: &str,
        secret: &str,
    ) -> (String, String, String) {
        let mut hasher = Sha256::new();
        let path_and_query = crate::http::UrlParser::get_path_and_query(url, host);
        let decoded_secret = base64::decode(secret).expect("secret is not valid base64");
        let mut mac = HmacSha256::new_varkey(&decoded_secret).expect("invalid hmac secret");
        let utc_now = Utc::now().to_rfc2822().replace("+0000", "GMT");
        let content_hash = match body {
            None => None,
            Some(b) => {
                hasher.input(b);
                Some(hasher.result())
            }
        };
        let content_hash_base64 = match content_hash {
            // use hard coded hash if no content to match javascript hashing of undefined
            None => String::from("47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="),
            Some(hash) => base64::encode(hash),
        };
        let string_to_sign = String::from(method)
            + "\n"
            + &path_and_query
            + "\n"
            + &utc_now
            + ";"
            + host
            + ";"
            + &content_hash_base64;

        mac.input(&string_to_sign.as_bytes());
        let hmac_result = mac.result().code();
        let signature = base64::encode(&hmac_result);
        let signed_headers = "x-ms-date;host;x-ms-content-sha256";
        (
            utc_now,
            content_hash_base64,
            String::from("HMAC-SHA256 Credential=")
                + credential
                + "&SignedHeaders="
                + signed_headers
                + "&Signature="
                + &signature,
        )
    }
}
