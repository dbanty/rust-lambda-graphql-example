# rust-lambda-graphql-example

A super simple example GraphQL API that can be hosted on AWS Lambda.

## What's in the box?

1. `src` contains the code to integrate the `async-graphql` crate with the `lambda` crates for a serverless GraphQL experience. Both POST and GET queries are supported on any path.
2. `.github` contains Renovate support for keeping dependencies up to date.
3. `.github/workflows` contains a GitHub Actions workflow that will build and test the code on every push as well as deploy on pushes to main. There is also an integration test which runs after deploy.

This is set up for the most simple use-case: a [function URL](https://docs.aws.amazon.com/lambda/latest/dg/lambda-urls.html). You will need to make some tweaks to get this working with API Gateway.

## To Set Up

1. Install [cargo-lambda](https://www.cargo-lambda.info/guide/getting-started.html)
2. Install [Zig](https://ziglang.org/download/) (needed by cargo-lambda)
3. An [AWS credentials file](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html) (for deploying only)

## How to use?

1. `cargo lambda watch` will start the service _lazily_ (only compiles when the first request comes in), then watch and reload your code on changes. The URL will be `http://localhost:9000/lambda-url/graphql-example` (where graphql-example is the name of the binary once you change it).
2. To deploy, first run `cargo lambda build --release --arm64` (for graviton processors) then `cargo lambda deploy`.
3. To get the GitHub Actions to deploy on push to main, you need to set two GitHub Secrets for AIM credentials: `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`.
4. For the integration tests to work, fill in the `FUNCTION_URL` secret with the URL of the deployed function. This assumes that the same AWS credentials which can deploy the function can invoke it—if this isn't true, you'll need to tweak the `integration_test` job in `.github/workflows/release.yml`. It also assumes that your function is secured with the AWS_IAM type of [function security](https://docs.aws.amazon.com/lambda/latest/dg/urls-auth.html). You'll need to tweak this job as soon as you change the schema.
