import * as core from "@aws-cdk/core";
import * as apigateway from "@aws-cdk/aws-apigateway";
import * as lambda from "@aws-cdk/aws-lambda";
import * as s3 from "@aws-cdk/aws-s3";

export class Service extends core.Construct {
    constructor(scope: core.Construct, id: string) {
        super(scope, id);

        const handler = new lambda.Function(this, "Function", {
            runtime: lambda.Runtime.PROVIDED_AL2,
            functionName: "GraphQLAPI",
            code: lambda.Code.fromAsset("bootstrap"),
            handler: "unused",
            timeout: core.Duration.seconds(30),
            environment: {
                DATABASE_URL: "postgres://postgres:local_password@host.docker.internal/postgres",
                RUST_LOG: "debug",
            }
        });


        new apigateway.LambdaRestApi(this, "API", {
            handler,
            restApiName: "GraphQL",
            description: "An GraphQL serverless app made with Rust's async-graphql framework.",
        });
    }
}