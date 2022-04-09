<div align="center">
  <h1><code>lottie-ast</code></h1>
  <p>
    <strong>A Lottie JSON file model library</strong>
  </p>
</div>

# lottie-ast
Hand-written models for Lottie's JSON schema, based on the marvelous documentation
of [Lottie Docs](https://lottiefiles.github.io/lottie-docs/).

To make it easier to understand, this library renames the fields of the schema.
If you prefer the original lottie namings, disable default feature and enable
the `keep-namings` feature.

Serialization/Deserialization are enabled via `serde` and `serde-json`.