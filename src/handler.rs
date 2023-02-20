use std::fmt::Display;

use async_graphql::{
    Request as GraphQlRequest, Response as GraphQlResponse, ServerError as GraphQlError,
};
use http::{Method, StatusCode};
use lambda_http::{Body, Error, Request, RequestExt, Response};

use crate::{
    errors::{ClientError, ServerError},
    schema::SCHEMA,
};

pub async fn handle_request(request: Request) -> Result<Response<Body>, Error> {
    let query = if request.method() == Method::POST {
        graphql_request_from_post(request)
    } else if request.method() == Method::GET {
        graphql_request_from_get(request)
    } else {
        Err(ClientError::MethodNotAllowed)
    };
    let query = match query {
        Err(e) => {
            return error_response(StatusCode::BAD_REQUEST, graphql_error(e));
        }
        Ok(query) => query,
    };
    let response_body =
        serde_json::to_string(&SCHEMA.execute(query).await).map_err(ServerError::from)?;
    Response::builder()
        .status(200)
        .body(Body::Text(response_body))
        .map_err(ServerError::from)
        .map_err(Error::from)
}

fn graphql_error(message: impl Display) -> String {
    let message = format!("{}", message);
    let response = GraphQlResponse::from_errors(vec![GraphQlError::new(message, None)]);
    serde_json::to_string(&response).expect("Valid response should never fail to serialize")
}

fn error_response(status: StatusCode, body: String) -> Result<Response<Body>, Error> {
    Ok(Response::builder().status(status).body(Body::Text(body))?)
}

fn graphql_request_from_post(request: Request) -> Result<GraphQlRequest, ClientError> {
    match request.into_body() {
        Body::Empty => Err(ClientError::EmptyBody),
        Body::Text(text) => {
            serde_json::from_str::<GraphQlRequest>(&text).map_err(ClientError::from)
        }
        Body::Binary(binary) => {
            serde_json::from_slice::<GraphQlRequest>(&binary).map_err(ClientError::from)
        }
    }
}

fn graphql_request_from_get(request: Request) -> Result<GraphQlRequest, ClientError> {
    let params = request.query_string_parameters();
    let query = params.first("query").ok_or(ClientError::MissingQuery)?;
    let mut request = async_graphql::Request::new(query);
    if let Some(operation_name) = params.first("operationName") {
        request = request.operation_name(operation_name)
    }
    if let Some(variables) = params.first("variables") {
        let value = serde_json::from_str(variables).unwrap_or_default();
        let variables = async_graphql::Variables::from_json(value);
        request = request.variables(variables);
    }
    Ok(request)
}
