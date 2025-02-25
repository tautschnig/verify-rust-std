//! Unstable module containing the unstable contracts lang items and attribute macros.

pub use crate::macros::builtin::{contracts_ensures as ensures, contracts_requires as requires};

/// Emitted by rustc as a desugaring of `#[ensures(PRED)] fn foo() -> R { ... [return R;] ... }`
/// into: `fn foo() { let _check = build_check_ensures(|ret| PRED) ... [return _check(R);] ... }`
/// (including the implicit return of the tail expression, if any).
///
/// This call helps with type inference for the predicate.
#[unstable(feature = "contracts_internals", issue = "128044" /* compiler-team#759 */)]
#[rustc_const_unstable(feature = "contracts", issue = "128044")]
#[lang = "contract_build_check_ensures"]
#[track_caller]
pub const fn build_check_ensures<Ret, C>(cond: C) -> C
where
    C: Fn(&Ret) -> bool + Copy + 'static,
{
    cond
}
