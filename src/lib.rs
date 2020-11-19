// SPDX-License-Identifier: Apache-2.0

//! Welcome to Ciborium!
//!
//! Ciborium contains utilities for working with CBOR types. This includes:
//!
//!   * Basic parsing (see `basic` module)
//!   * Serde serialization/deserialization (see `serde` module)
//!   * Tokio frame codec for [CBOR sequences](https://tools.ietf.org/html/rfc8742)
//!     (see `tokio` module)
//!
//! Ciborium has the following feature flags:
//!
//!   * `serde` - enables limited `serde` support (i.e. `no_std`)
//!   * `std`   - enables complete `serde` support (implies `serde` flag)
//!   * `tokio` - enables `tokio` support (implies `std` flag)

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(missing_docs)]
#![deny(clippy::cargo)]

extern crate alloc;

mod io;

pub mod basic;
pub mod value;

#[cfg(feature = "serde")]
pub mod de;

#[cfg(feature = "serde")]
pub mod ser;

/// Build a `Value` conveniently.
///
/// The syntax should be intuitive if you are familiar with JSON. You can also
/// inline simple Rust expressions, including custom values that implement
/// `serde::Serialize`. Note that this macro returns `Result<Value, Error>`,
/// so you should handle the error appropriately.
///
/// ```
/// use ciborium::cbor;
///
/// let value = cbor!({
///     "code" => 415,
///     "message" => null,
///     "continue" => false,
///     "extra" => { "numbers" => [8.2341e+4, 0.251425] },
/// }).unwrap();
/// ```
#[macro_export]
#[cfg(feature = "serde")]
macro_rules! cbor {
    (@map {$($key:expr => $val:expr),*} $(,)?) => {{
        $crate::value::Value::Map(vec![
            $(
                (cbor!( $key )?, cbor!( $val )?)
            ),*
        ])
    }};

    (@map {$($key:expr => $val:expr),*} { $($nkey:tt)* } => $($next:tt)*) => {
        cbor!(
            @map
            { $($key => $val),* }
            cbor!({ $($nkey)* })? =>
            $($next)*
        )
    };

    (@map {$($key:expr => $val:expr),*} [ $($nkey:tt)* ] => $($next:tt)*) => {
        cbor!(
            @map
            { $($key => $val),* }
            cbor!([ $($nkey)* ])? =>
            $($next)*
        )
    };

    (@map {$($key:expr => $val:expr),*} $nkey:expr => { $($nval:tt)* }, $($next:tt)*) => {
        cbor!(
            @map
            { $($key => $val,)* $nkey => cbor!({ $($nval)* })? }
            $($next)*
        )
    };

    (@map {$($key:expr => $val:expr),*} $nkey:expr => [ $($nval:tt)* ], $($next:tt)*) => {
        cbor!(
            @map
            { $($key => $val,)* $nkey => cbor!([ $($nval)* ])? }
            $($next)*
        )
    };

    (@map {$($key:expr => $val:expr),*} $nkey:expr => $nval:expr, $($next:tt)*) => {
        cbor!(
            @map
            { $($key => $val,)* $nkey => cbor!($nval)? }
            $($next)*
        )
    };

    (@seq [$($val:expr),*] $(,)?) => {
        $crate::value::Value::Array(
            vec![$( cbor!($val)? ),*]
        )
    };

    (@seq [$($val:expr),*] { $($item:tt)* }, $($next:tt)*) => {
        cbor!(
            @seq
            [ $($val,)* cbor!({ $($item)* })? ]
            $($next)*
        )
    };

    (@seq [$($val:expr),*] [ $($item:tt)* ], $($next:tt)*) => {
        cbor!(
            @seq
            [ $($val,)* cbor!([ $($item)* ])? ]
            $($next)*
        )
    };

    (@seq [$($val:expr),*] $item:expr, $($next:tt)*) => {
        cbor!(
            @seq
            [ $($val,)* $item ]
            $($next)*
        )
    };

    ({ $($next:tt)* }) => {(||{
        ::core::result::Result::<_, $crate::value::Error>::from(Ok(cbor!(@map {} $($next)* ,)))
    })()};

    ([ $($next:tt)* ]) => {(||{
        ::core::result::Result::<_, $crate::value::Error>::from(Ok(cbor!(@seq [] $($next)* ,)))
    })()};

    ($val:expr) => {{
        #[allow(unused_imports)]
        use $crate::value::Value::Null as null;
        $crate::value::Value::serialized(&$val)
    }};
}
