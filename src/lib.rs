use {
    failure::Fallible,
    futures::prelude::*,
    headers::*,
    http::{HttpTryFrom, Method, Response},
    log::*,
    serde::{Deserialize, Serialize},
};

pub trait HttpClientExt: Sized {
    fn create() -> Self;
    fn build_request(&self) -> RequestBuilder;
}

pub type HttpClient = hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>;

#[derive(Debug)]
pub struct RequestBuilder {
    pub http_client: HttpClient,
    pub request: http::Request<hyper::Body>,
}

impl HttpClientExt for HttpClient {
    fn create() -> Self {
        hyper::Client::builder().build(hyper_tls::HttpsConnector::new().unwrap())
    }

    fn build_request(&self) -> RequestBuilder {
        RequestBuilder {
            http_client: self.clone(),
            request: http::Request::default(),
        }
    }
}

impl std::ops::Deref for RequestBuilder {
    type Target = http::Request<hyper::Body>;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}
impl std::ops::DerefMut for RequestBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.request
    }
}

impl RequestBuilder {
    pub fn method(mut self, method: Method) -> Self {
        *self.method_mut() = method;
        self
    }
    pub fn header<H: headers::Header>(mut self, header: H) -> Self {
        self.headers_mut().typed_insert(header);
        self
    }
    pub fn uri(mut self, uri: &str) -> Fallible<Self> {
        *self.uri_mut() = HttpTryFrom::try_from(uri)?;
        Ok(self)
    }
    pub fn body_json<B: Serialize>(mut self, body: B) -> Fallible<Self> {
        *self.body_mut() = serde_json::to_string(&body)?.into();
        self.headers_mut()
            .typed_insert(headers::ContentType::json());

        Ok(self)
    }
    pub async fn recv(self) -> Fallible<Response<hyper::Body>> {
        Ok(self.http_client.request(self.request).await?)
    }
    pub async fn recv_json<T: for<'de> Deserialize<'de>>(self) -> Fallible<T> {
        debug!("Sending request: {:?}", self.request);

        let rsp = self.http_client.request(self.request).await?;

        let rsp_dbg = format!("{:?}", rsp);

        let body = String::from_utf8(rsp.into_body().try_concat().await?.to_vec())?;

        debug!("Received response: {} with body {}", rsp_dbg, body);

        Ok(serde_json::from_str::<T>(&body)?)
    }
}
