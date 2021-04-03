import * as cdk from '@aws-cdk/core';
import {Service} from "./service";

export class CdkStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    new Service(this, "Service");
  }
}
