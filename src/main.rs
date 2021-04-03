use std::future::Future;
use std::pin::Pin;

use async_graphql::{EmptyMutation, EmptySubscription, Object, Request as GraphQLRequest, Schema};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequestContext, ApiGatewayV2httpRequestContext};
use http::{Method, StatusCode};
use lamedh_http::{Context, Error as LambdaError, handler, Handler, Request, RequestExt, Response};
use lamedh_http::request::RequestContext;
use thiserror::Error;

#[derive(Error, Debug)]
enum InternalServerError {
    #[error("could not serialize JSON")]
    Disconnect(#[from] serde_json::Error),
    #[error("error creating response")]
    Response(#[from] http::Error),
}

#[derive(Error, Debug)]
enum BadRequestError {
    #[error(transparent)]
    Query(#[from] async_graphql::ParseRequestError),
    #[error(transparent)]
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

struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

struct ApiHandler {
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
}

impl ApiHandler {
    fn new() -> ApiHandler {
        ApiHandler {
            schema: Schema::new(Query, EmptyMutation, EmptySubscription),
        }
    }
}

type HandlerError = InternalServerError;
type HandlerResponse = Response<String>;
type HandlerResult = Result<HandlerResponse, HandlerError>;

impl Handler for ApiHandler {
    type Error = InternalServerError;
    type Response = HandlerResponse;

    type Fut = Pin<Box<dyn Future<Output = HandlerResult> + 'static>>;

    fn call(&mut self, req: Request, _ctx: Context) -> Self::Fut {
        let schema = self.schema.clone();
        let fut = async move { handle_request(schema, req).await };
        Box::pin(fut)
    }
}

async fn handle_request(
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
    request: Request,
) -> Result<HandlerResponse, InternalServerError> {
    match request.uri().path() {
        "/" => Response::builder()
            .header("Content-Type", "text/html")
            .body(async_graphql::http::graphiql_source(&graphql_endpoint_with_stage(request.request_context()), None))
            .map_err(InternalServerError::from),
        "/graphql" => handle_query(schema, request).await,
        _ => error_response(http::StatusCode::NOT_FOUND, "Invalid path".to_string()),
    }
}

fn graphql_endpoint_with_stage(request_context: RequestContext) -> String {
    let stage = match request_context {
        RequestContext::ApiGatewayV1(ApiGatewayProxyRequestContext {
                                         stage,
                                         ..
                                     }) => stage,
        RequestContext::ApiGatewayV2(ApiGatewayV2httpRequestContext {
                                         stage,
                                         ..
                                     }) => stage,
        RequestContext::Alb(..) => None,
    };
    match stage {
        Some(stage) => format!("/{}/graphql", stage),
        None => "/graphql".to_string()
    }
}

async fn handle_query(
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
    request: Request,
) -> Result<HandlerResponse, InternalServerError> {
    let query = if request.method() == Method::POST {
        graphql_request_from_post(request)
    } else if request.method() == Method::GET {
        graphql_request_from_get(request)
    } else {
        Err(BadRequestError::MethodNotAllowed)
    };
    let query = match query {
        Err(e) => {
            return error_response(StatusCode::BAD_REQUEST, format!("{}", e));
        }
        Ok(query) => query,
    };
    let response_body =
        serde_json::to_string(&schema.execute(query).await).map_err(InternalServerError::from)?;
    Response::builder()
        .status(200)
        .body(response_body)
        .map_err(|e| e.into())
}

fn error_response(
    status: http::StatusCode,
    body: String,
) -> Result<Response<String>, InternalServerError> {
    Response::builder()
        .status(status)
        .body(body)
        .map_err(InternalServerError::from)
}

fn graphql_request_from_post(request: Request) -> Result<GraphQLRequest, BadRequestError> {
    match request.into_body() {
        Body::Empty => Err(BadRequestError::EmptyBody),
        Body::Text(text) => {
            serde_json::from_str::<GraphQLRequest>(&text).map_err(BadRequestError::from)
        }
        Body::Binary(binary) => {
            serde_json::from_slice::<GraphQLRequest>(&binary).map_err(BadRequestError::from)
        }
    }
}

fn graphql_request_from_get(request: Request) -> Result<GraphQLRequest, BadRequestError> {
    let params = request.query_string_parameters();
    let query = params.get("query").ok_or(BadRequestError::MissingQuery)?;
    let mut request = async_graphql::Request::new(query);
    if let Some(operation_name) = params.get("operationName") {
        request = request.operation_name(operation_name)
    }
    if let Some(variables) = params.get("variables") {
        let value = serde_json::from_str(&variables).unwrap_or_default();
        let variables = async_graphql::Variables::from_json(value);
        request = request.variables(variables);
    }
    Ok(request)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), LambdaError> {
    lamedh_runtime::run(handler(ApiHandler::new())).await?;
    Ok(())
}
