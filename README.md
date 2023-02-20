# rust-lambda-graphql-example

A super simple example GraphQL API that can be hosted on AWS Lambda.

## What's in the box?

1. `src` contains the code to integrate the `async-graphql` crate with the `lamedh` crates for a serverless GraphQL experience. You can hit the root `/` path to see a `GraphiQL` UI or the `/graphql` path to make queries.
2. The `cdk` directory contains all the infrastructure as code, `cdk/cdk.ts` is where you'll want to tweak your own 
   infrastructure.
3. `Makefile.toml` contains a couple useful rules for working with this project (uses `cargo-make`).
4. `docker-compose.yml` contains a basic Postgres setup that all the rules in this project use for testing/running.

## To Set Up (macOS)

1. Install node: `brew install node`
2. Install node dependencies: `npm install --prefix cdk`
3. Install cargo-make: `cargo install cargo-make`
4. Install AWS SAM CLI for local development: `pipx install aws-sam-cli` (or `pip` or `brew` if you prefer)
5. Setup MUSL cross-compile toolchain (since that's what lambda runtime needs):
    1. Add MUSL target: `rustup target add x86_64-unknown-linux-musl`
    2. Install MUSL cross-compile tool: `brew install FiloSottile/musl-cross/musl-cross`
    3. Soft link musl-gcc: `ln -s /usr/local/bin/x86_64-linux-musl-gcc /usr/local/bin/musl-gcc`

## How to use?

1. Use `makers local` to run the API locally. This will use CDK to generate a template for SAM CLI, then use SAM CLI to 
  run locally. **This rule requires Docker and SAM CLI to be installed**.
2. Use `makers deploy` to deploy to AWS with your default profile. You have to have AWS credentials set up. **Modify package.json or Makefile.toml if you want to use a different profile**.
    1. If you have not used `npm exec --prefix cdk cdk bootstrap` before on you account, you'll need to run this once.
    2. Use `npm exec --prefix cdk cdk destroy` to tear down the API. You may have to find and empty the generated S3 bucket manually for this to work. This will not destroy the stack created by `bootstrap`, you have to do that yourself in CloudFormation if you want it gone.
