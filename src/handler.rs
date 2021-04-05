use std::future::Future;
use std::pin::Pin;

use async_graphql::{
    ObjectType, Request as GraphQlRequest, Response as GraphQlResponse, Schema,
    ServerError as GraphQlError, SubscriptionType,
};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::apigw::{
    ApiGatewayProxyRequestContext, ApiGatewayV2httpRequestContext,
};
use http::{Method, StatusCode};
use lamedh_http::request::RequestContext;
use lamedh_http::{Context, Handler, Request, RequestExt, Response};

use crate::errors::{ClientError, ServerError};
use std::fmt::Display;

pub(crate) struct GraphQlHandler<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    schema: Schema<Query, Mutation, Subscription>,
}

impl<Query, Mutation, Subscription> GraphQlHandler<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    pub fn new(
        schema: Schema<Query, Mutation, Subscription>,
    ) -> GraphQlHandler<Query, Mutation, Subscription>
    where
        Query: ObjectType + 'static,
        Mutation: ObjectType + 'static,
        Subscription: SubscriptionType + 'static,
    {
        GraphQlHandler { schema }
    }
}

type HandlerError = ServerError;
type HandlerResponse = Response<String>;
type HandlerResult = Result<HandlerResponse, HandlerError>;

impl<Query, Mutation, Subscription> Handler for GraphQlHandler<Query, Mutation, Subscription>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    type Error = ServerError;
    type Response = HandlerResponse;

    type Fut = Pin<Box<dyn Future<Output = HandlerResult> + 'static>>;

    fn call(&mut self, req: Request, _ctx: Context) -> Self::Fut {
        let schema = self.schema.clone();
        let fut = async move { handle_request(schema, req).await };
        Box::pin(fut)
    }
}

async fn handle_request<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
    request: Request,
) -> Result<HandlerResponse, ServerError>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
    let path = request.uri().path();
    if path == "/" {
        Response::builder()
            .header("Content-Type", "text/html")
            .body(async_graphql::http::graphiql_source(
                &graphql_endpoint_with_stage(request.request_context()),
                None,
            ))
            .map_err(ServerError::from)
    } else if path.ends_with("/graphql") {
        handle_query(schema, request).await
    } else {
        error_response(http::StatusCode::NOT_FOUND, "Invalid path".to_string())
    }
}

fn graphql_endpoint_with_stage(request_context: RequestContext) -> String {
    let stage = match request_context {
        RequestContext::ApiGatewayV1(ApiGatewayProxyRequestContext { stage, .. }) => stage,
        RequestContext::ApiGatewayV2(ApiGatewayV2httpRequestContext { stage, .. }) => stage,
        RequestContext::Alb(..) => None,
    };
    match stage {
        Some(stage) => format!("/{}/graphql", stage),
        None => "/graphql".to_string(),
    }
}

async fn handle_query<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
    request: Request,
) -> Result<HandlerResponse, ServerError>
where
    Query: ObjectType + 'static,
    Mutation: ObjectType + 'static,
    Subscription: SubscriptionType + 'static,
{
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
        serde_json::to_string(&schema.execute(query).await).map_err(ServerError::from)?;
    Response::builder()
        .status(200)
        .body(response_body)
        .map_err(|e| e.into())
}

fn graphql_error(message: impl Display) -> String {
    let message = format!("{}", message);
    let response = GraphQlResponse::from_errors(vec![GraphQlError::new(message)]);
    serde_json::to_string(&response).expect("Valid response should never fail to serialize")
}

fn error_response(status: http::StatusCode, body: String) -> Result<Response<String>, ServerError> {
    Response::builder()
        .status(status)
        .body(body)
        .map_err(ServerError::from)
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
    let query = params.get("query").ok_or(ClientError::MissingQuery)?;
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
