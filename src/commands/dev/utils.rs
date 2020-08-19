use http::{HeaderValue, Response};
use hyper::{Body, Uri};
use url::Url;

pub(super) fn get_path_as_str(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

pub(super) fn rewrite_redirect(resp: &mut Response<Body>, upstream_host: &str, local_host: &str) {
    if resp.status().is_redirection() {
        let headers = resp.headers_mut();
        if let Some(destination) = headers.get("Location") {
            println!("got location");
            if let Ok(destination) = destination.to_str() {
                println!("got destination str");
                if let Ok(destination_url) = Url::parse(destination) {
                    println!("got destination url");
                    if let Some(destination_domain) = destination_url.domain() {
                        println!("got destination domain");
                        if destination_domain == upstream_host {
                            println!("destination domain is upstream host");
                            let rewritten_destination =
                                destination.replace(destination_domain, local_host);
                            println!("rewriting to {:?}", &rewritten_destination);
                            if let Ok(header_value) = HeaderValue::from_str(&rewritten_destination)
                            {
                                headers.insert("Location", header_value);
                            }
                        }
                    }
                }
            }
        }
    }
}
