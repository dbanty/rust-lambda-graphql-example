#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';
import * as apigateway from "@aws-cdk/aws-apigateway";
import * as lambda from "@aws-cdk/aws-lambda";

export class Service extends cdk.Construct {
    constructor(scope: cdk.Construct, id: string) {
        super(scope, id);

        const handler = new lambda.Function(this, "Function", {
            runtime: lambda.Runtime.PROVIDED_AL2,
            functionName: "GraphQLAPI",
            code: lambda.Code.fromAsset("../bootstrap"),
            handler: "unused",
            timeout: cdk.Duration.seconds(5),
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

export class CdkStack extends cdk.Stack {
    constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
        super(scope, id, props);

        new Service(this, "Service");
    }
}

const app = new cdk.App();
new CdkStack(app, 'CdkStack');
