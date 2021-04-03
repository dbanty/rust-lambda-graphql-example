# rust-lambda-graphql-example

A super simple example GraphQL API that can be hosted on AWS Lambda.

## What's in the box?

1. `src/main.rs` contains the code to integrate the `async-graphql` crate with the `lamedh` crates for a serverless GraphQL experience. You can hit the root `/` path to see a `GraphiQL` UI or the `/graphql` path to make queries.
2. There are a bunch of files required to set up CDK to manage the infrastructure / deployment of the function. You'll want to modify `cdk/lib/service.ts` to name your API and add any other infrastructure (e.g. a custom URL with route53).
3. `Makefile` contains a couple useful rules for working with this project.

## To Set Up (macOS)

1. Install node: `brew install node`
1. Install node dependencies: `npm install`
1. Add MUSL target: `rustup target add x86_64-unknown-linux-musl`
1. Install MUSL cross-compile tool: `brew install FiloSottile/musl-cross/musl-cross`
1. Soft link musl-gcc: `ln -s /usr/local/bin/x86_64-linux-musl-gcc /usr/local/bin/musl-gcc`

## How to use?

1. Use `make local` to run the API locally. This will use CDK to generate a template for SAM CLI, then use SAM CLI to 
  run locally. **This rule requires Docker and SAM CLI to be installed**.
1. Use `make deploy` to deploy to AWS with your default profile. You have to have AWS credentials set up. **Modify package.json if you want to use a different profile**.
    1. If you have not used `npm exec cdk bootstrap` before on you account, you'll need to run this once.
    1. Use `npm exec cdk destroy` to tear down the API. You may have to find and empty the generated S3 bucket manually for this to work. This will not destroy the stack created by `bootstrap`, you have to do that yourself in CloudFormation if you want it gone.
