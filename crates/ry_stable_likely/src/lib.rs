//! # Branch prediction optimization
//!
//! If you are sure that one branch is more likely to be taken than the other,
//! you can use the [`likely`] and [`unlikely`].
//!
//! * This is a stable replacement for the [`intrinsics::likely`] and [`intrinsics::unlikely`].
//!
//! [`intrinsics::likely`]: https://doc.rust-lang.org/std/intrinsics/fn.likely.html
//! [`intrinsics::unlikely`]: https://doc.rust-lang.org/std/intrinsics/fn.unlikely.html

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::unnested_or_patterns
)]

#[inline]
#[cold]
const fn cold() {}

/// The function allows to tell the compiler that the condition is likely to be
/// `true`.
#[inline]
#[must_use]
pub const fn likely(b: bool) -> bool {
    // If `b` is `false`, it calls the `cold()` function. The purpose of calling `cold()`
    // in this case is to potentially hint to the compiler that the code path
    // where `b` is false is unlikely to be taken frequently. After that, the
    // function returns the value of `b`.
    if !b {
        cold();
    }

    b
}

/// The function allows to tell the compiler that the condition is unlikely to be
/// `true`.
#[inline]
#[must_use]
pub const fn unlikely(b: bool) -> bool {
    // It checks if `b` is true instead. If b is true, it calls the `cold()` function.
    // Again, the purpose is to potentially hint to the compiler that the code path
    // where `b` is `true` is unlikely to be taken frequently.
    if b {
        cold();
    }

    b
}
