[![Workflow Status](https://github.com/enarx/ciborium/workflows/test/badge.svg)](https://github.com/enarx/ciborium/actions?query=workflow%3A%22test%22)
[![Average time to resolve an issue](https://isitmaintained.com/badge/resolution/enarx/ciborium.svg)](https://isitmaintained.com/project/enarx/ciborium "Average time to resolve an issue")
[![Percentage of issues still open](https://isitmaintained.com/badge/open/enarx/ciborium.svg)](https://isitmaintained.com/project/enarx/ciborium "Percentage of issues still open")
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# ciborium

Welcome to Ciborium!

Ciborium contains utilities for working with CBOR types. This includes:

  * Basic parsing (see `basic` module)
  * Serde serialization/deserialization (see `serde` module)
  * Tokio frame codec for [CBOR sequences](https://tools.ietf.org/html/rfc8742)
    (see `tokio` module)

Ciborium has the following feature flags:

  * `serde` - enables limited `serde` support (i.e. `no_std`)
  * `std`   - enables complete `serde` support (implies `serde` flag)
  * `tokio` - enables `tokio` support (implies `std` flag)

License: Apache-2.0
