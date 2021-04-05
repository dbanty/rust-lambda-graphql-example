use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ServerError {
    #[error("could not serialize JSON")]
    Disconnect(#[from] serde_json::Error),
    #[error("error creating response")]
    Response(#[from] http::Error),
}

#[derive(Error, Debug)]
pub(crate) enum ClientError {
    #[error(transparent)]
    Query(#[from] async_graphql::ParseRequestError),
    #[error("Could not parse JSON body")]
    Json(#[from] serde_json::Error),
    #[error("POST request must contain a body")]
    EmptyBody,
    #[error("Binary body must be encoded with UTF-8")]
    InvalidBinaryBody(#[from] std::str::Utf8Error),
    #[error("Only GET and POST methods are allowed")]
    MethodNotAllowed,
    #[error("query param is required with GET method")]
    MissingQuery,
}
