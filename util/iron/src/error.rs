use std::fmt;

use failure::err_msg;
use failure::Fail;
use iron::headers::ContentType;
use iron::status;
use iron::IronError;
use iron::Response;
use iron::Set;
use iron_json_response::JsonResponse;
use opentracingrust::Error as OTError;
use serde_json;

use replicante_util_failure::SerializableFail;

/// Helper function to convert a `Fail` into an `IronError`.
///
/// The iron `Response` attached to this error returns a JSON serialised `SerializableFail`.
pub fn into_ironerror<E: Fail>(error: E) -> IronError {
    let display = error.to_string();
    let wrapper = SerializableFail::from(error);
    let mut response = Response::with((
        status::InternalServerError,
        serde_json::to_string(&wrapper).unwrap(),
    ));
    response.headers.set(ContentType::json());
    let error = Box::new(ErrorWrapper { display });
    IronError { error, response }
}

/// Convert an OpenTracingRust error into an IronError.
#[allow(clippy::needless_pass_by_value)]
pub fn otr_into_ironerror(error: OTError) -> IronError {
    let error = format!("{:?}", error);
    let wrapper = SerializableFail {
        error: error.clone(),
        layers: vec![error.clone()],
        trace: None,
    };
    let mut response = Response::new();
    response
        .set_mut(JsonResponse::json(wrapper))
        .set_mut(status::BadRequest);
    // OTError should really have implemented `Error` :-(
    let error = err_msg(error).compat();
    IronError {
        error: Box::new(error),
        response,
    }
}

/// Internal compatibility type between a `Fail` and an `iron::Error`.
#[derive(Debug)]
struct ErrorWrapper {
    display: String,
}

impl fmt::Display for ErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.display, f)
    }
}

impl ::iron::Error for ErrorWrapper {
    fn description(&self) -> &str {
        &self.display
    }
}

#[cfg(test)]
mod tests {
    use failure::err_msg;
    use failure::Fail;

    use iron::headers::ContentType;
    use iron::Headers;
    use iron::IronError;
    use iron::IronResult;
    use iron::Request;
    use iron::Response;
    use iron_test::request;
    use iron_test::response;

    use super::into_ironerror;

    fn failing(_: &mut Request) -> IronResult<Response> {
        let error = err_msg("test").context("chained").context("failures");
        let error: IronError = into_ironerror(error);
        Err(error)
    }

    #[test]
    fn error_conversion() {
        let response = request::get("http://host:16016/", Headers::new(), &failing);
        let response = match response {
            Err(error) => error.response,
            Ok(_) => panic!("Request should fail"),
        };

        let content_type = response.headers.get::<ContentType>().unwrap().clone();
        assert_eq!(content_type, ContentType::json());

        let result_body = response::extract_body_to_bytes(response);
        let result_body = String::from_utf8(result_body).unwrap();
        assert_eq!(
            result_body,
            r#"{"error":"failures","layers":["failures","chained","test"],"trace":null}"#,
        );
    }
}
