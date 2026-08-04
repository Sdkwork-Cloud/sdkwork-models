# SDKWork Models SDKs

This directory is the SDK boundary owned by `sdkwork-models`.

- `sdkwork-models-sdk` owns the provider-standard file catalog SDK workspace for loading,
  validating, and querying the portable model catalog across TypeScript, Python, Java, Rust, and
  Flutter.

Catalog HTTP APIs, database import, and RPC services are owned by consumer applications such as
CloudRouter. This repository intentionally does not publish OpenAPI-generated HTTP SDK families.
