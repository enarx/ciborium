//! Canonicalization support for CBOR serialization.
//!
//! Supports various canonicalization schemes for deterministic CBOR serialization. The default is
//! [NoCanonicalization] for the fastest serialization. Canonical serialization is around 2x slower.

/// Which canonicalization scheme to use for CBOR serialization.
///
/// Can only be initialized with the `std` feature enabled.
#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CanonicalizationScheme {
    /// Sort map keys in output according to [RFC 7049]'s deterministic encoding spec.
    ///
    /// Also aligns with [RFC 8949 4.2.3]'s backwards compatibility sort order.
    ///
    /// Uses length-first map key ordering. Eg. `["a", "b", "aa"]`.
    #[cfg(feature = "std")]
    Rfc7049,

    /// Sort map keys in output according to [RFC 8949]'s deterministic encoding spec.
    ///
    /// Uses bytewise lexicographic map key ordering. Eg. `["a", "aa", "b"]`.
    #[cfg(feature = "std")]
    Rfc8949,
}

/// Don't sort map key output.
pub struct NoCanonicalization;

/// Sort map keys in output according to [RFC 7049]'s deterministic encoding spec.
///
/// Also aligns with [RFC 8949 4.2.3]'s backwards compatibility sort order.
///
/// Uses length-first map key ordering. Eg. `["a", "b", "aa"]`.
#[cfg(feature = "std")]
pub struct Rfc7049;

/// Sort map keys in output according to [RFC 8949]'s deterministic encoding spec.
///
/// Uses bytewise lexicographic map key ordering. Eg. `["a", "aa", "b"]`.
#[cfg(feature = "std")]
pub struct Rfc8949;

/// Trait for canonicalization schemes.
///
/// See implementors:
/// - [NoCanonicalization] for no canonicalization (fastest).
/// - [Rfc7049] for length-first map key sorting.
/// - [Rfc8949] for bytewise lexicographic map key sorting.
pub trait Canonicalization {
    /// True if keys should be cached and sorted.
    const IS_CANONICAL: bool;

    /// Determines which sorting implementation to use.
    const SCHEME: Option<CanonicalizationScheme>;
}

impl Canonicalization for NoCanonicalization {
    const IS_CANONICAL: bool = false;
    const SCHEME: Option<CanonicalizationScheme> = None;
}

#[cfg(feature = "std")]
impl Canonicalization for Rfc7049 {
    const IS_CANONICAL: bool = true;
    const SCHEME: Option<CanonicalizationScheme> = Some(CanonicalizationScheme::Rfc7049);
}

#[cfg(feature = "std")]
impl Canonicalization for Rfc8949 {
    const IS_CANONICAL: bool = true;
    const SCHEME: Option<CanonicalizationScheme> = Some(CanonicalizationScheme::Rfc8949);
}
