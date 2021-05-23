use crate::ops::ControlFlow;

/// The `?` operator and `try {}` blocks.
///
/// `try_*` methods typically involve a type implementing this trait.  For
/// example, the closures passed to [`Iterator::try_fold`] and
/// [`Iterator::try_for_each`] must return such a type.
///
/// `Try` types are typically those containing two or more categories of values,
/// some subset of which are so commonly handled via early returns that it's
/// worth providing a terse (but still visible) syntax to make that easy.
///
/// This is most often seen for error handling with [`Result`] and [`Option`].
/// The quintessential implementation of this trait is on [`ControlFlow`].
///
/// # Using `Try` in Generic Code
///
/// `Iterator::try_fold` was stabilized to call back in Rust 1.27, but
/// this trait is much newer.  To illustrate the various associated types and
/// methods, let's implement our own version.
///
/// As a reminder, an infallible version of a fold looks something like this:
/// ```
/// fn simple_fold<A, T>(
///     iter: impl Iterator<Item = T>,
///     mut accum: A,
///     mut f: impl FnMut(A, T) -> A,
/// ) -> A {
///     for x in iter {
///         accum = f(accum, x);
///     }
///     accum
/// }
/// ```
///
/// So instead of `f` returning just an `A`, we'll need it to return some other
/// type that produces an `A` in the "don't short circuit" path.  Conveniently,
/// that's also the type we need to return from the function.
///
/// Let's add a new generic parameter `R` for that type, and bound it to the
/// output type that we want:
/// ```
/// # #![feature(try_trait_v2)]
/// # use std::ops::Try;
/// fn simple_try_fold_1<A, T, R: Try<Output = A>>(
///     iter: impl Iterator<Item = T>,
///     mut accum: A,
///     mut f: impl FnMut(A, T) -> R,
/// ) -> R {
///     todo!()
/// }
/// ```
///
/// If we get through the entire iterator, we need to wrap up the accumulator
/// into the return type using [`Try::from_output`]:
/// ```
/// # #![feature(try_trait_v2)]
/// # use std::ops::{ControlFlow, Try};
/// fn simple_try_fold_2<A, T, R: Try<Output = A>>(
///     iter: impl Iterator<Item = T>,
///     mut accum: A,
///     mut f: impl FnMut(A, T) -> R,
/// ) -> R {
///     for x in iter {
///         let cf = f(accum, x).branch();
///         match cf {
///             ControlFlow::Continue(a) => accum = a,
///             ControlFlow::Break(_) => todo!(),
///         }
///     }
///     R::from_output(accum)
/// }
/// ```
///
/// We'll also need [`FromResidual::from_residual`] to turn the residual back
/// into the original type.  But because it's a supertrait of `Try`, we don't
/// need to mention it in the bounds.  All types which implement `Try` can be
/// recreated from their corresponding residual, so we'll just call it:
/// ```
/// # #![feature(try_trait_v2)]
/// # use std::ops::{ControlFlow, Try};
/// pub fn simple_try_fold_3<A, T, R: Try<Output = A>>(
///     iter: impl Iterator<Item = T>,
///     mut accum: A,
///     mut f: impl FnMut(A, T) -> R,
/// ) -> R {
///     for x in iter {
///         let cf = f(accum, x).branch();
///         match cf {
///             ControlFlow::Continue(a) => accum = a,
///             ControlFlow::Break(r) => return R::from_residual(r),
///         }
///     }
///     R::from_output(accum)
/// }
/// ```
///
/// But this "call `branch`, then `match` on it, and `return` if it was a
/// `Break`" is exactly what happens inside the `?` operator.  So rather than
/// do all this manually, we can just use `?` instead:
/// ```
/// # #![feature(try_trait_v2)]
/// # use std::ops::Try;
/// fn simple_try_fold<A, T, R: Try<Output = A>>(
///     iter: impl Iterator<Item = T>,
///     mut accum: A,
///     mut f: impl FnMut(A, T) -> R,
/// ) -> R {
///     for x in iter {
///         accum = f(accum, x)?;
///     }
///     R::from_output(accum)
/// }
/// ```
#[unstable(feature = "try_trait_v2", issue = "84277")]
#[rustc_on_unimplemented(
    on(
        all(from_method = "from_output", from_desugaring = "TryBlock"),
        message = "a `try` block must return `Result` or `Option` \
                    (or another type that implements `{Try}`)",
        label = "could not wrap the final value of the block as `{Self}` doesn't implement `Try`",
    ),
    on(
        all(from_method = "branch", from_desugaring = "QuestionMark"),
        message = "the `?` operator can only be applied to values \
                    that implement `{Try}`",
        label = "the `?` operator cannot be applied to type `{Self}`"
    )
)]
#[doc(alias = "?")]
#[cfg_attr(not(bootstrap), lang = "Try")]
pub trait Try: FromResidual {
    /// The type of the value produced by `?` when *not* short-circuiting.
    #[unstable(feature = "try_trait_v2", issue = "84277")]
    type Output;

    /// The type of the value passed to [`FromResidual::from_residual`]
    /// as part of `?` when short-circuiting.
    ///
    /// This represents the possible values of the `Self` type which are *not*
    /// represented by the `Output` type.
    ///
    /// # Note to Implementors
    ///
    /// The choice of this type is critical to interconversion.
    /// Unlike the `Output` type, which will often be a raw generic type,
    /// this type is typically a newtype of some sort to "color" the type
    /// so that it's distinguishable from the residuals of other types.
    ///
    /// This is why `Result<T, E>::Residual` is not `E`, but `Result<Infallible, E>`.
    /// That way it's distinct from `ControlFlow<E>::Residual`, for example,
    /// and thus `?` on `ControlFlow` cannot be used in a method returning `Result`.
    ///
    /// If you're making a generic type `Foo<T>` that implements `Try<Output = T>`,
    /// then typically you can use `Foo<std::convert::Infallible>` as its `Residual`
    /// type: that type will have a "hole" in the correct place, and will maintain the
    /// "foo-ness" of the residual so other types need to opt-in to interconversion.
    #[unstable(feature = "try_trait_v2", issue = "84277")]
    type Residual;

    /// Constructs the type from its `Output` type.
    ///
    /// This should be implemented consistently with the `branch` method
    /// such that applying the `?` operator will get back the original value:
    /// `Try::from_output(x).branch() --> ControlFlow::Continue(x)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(try_trait_v2)]
    /// use std::ops::Try;
    ///
    /// assert_eq!(<Result<_, String> as Try>::from_output(3), Ok(3));
    /// assert_eq!(<Option<_> as Try>::from_output(4), Some(4));
    /// assert_eq!(
    ///     <std::ops::ControlFlow<String, _> as Try>::from_output(5),
    ///     std::ops::ControlFlow::Continue(5),
    /// );
    ///
    /// # fn make_question_mark_work() -> Option<()> {
    /// assert_eq!(Option::from_output(4)?, 4);
    /// # None }
    /// # make_question_mark_work();
    ///
    /// // This is used, for example, on the accumulator in `try_fold`:
    /// let r = std::iter::empty().try_fold(4, |_, ()| -> Option<_> { unreachable!() });
    /// assert_eq!(r, Some(4));
    /// ```
    #[cfg_attr(not(bootstrap), lang = "from_output")]
    #[unstable(feature = "try_trait_v2", issue = "84277")]
    fn from_output(output: Self::Output) -> Self;

    /// Used in `?` to decide whether the operator should produce a value
    /// (because this returned [`ControlFlow::Continue`])
    /// or propagate a value back to the caller
    /// (because this returned [`ControlFlow::Break`]).
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(try_trait_v2)]
    /// use std::ops::{ControlFlow, Try};
    ///
    /// assert_eq!(Ok::<_, String>(3).branch(), ControlFlow::Continue(3));
    /// assert_eq!(Err::<String, _>(3).branch(), ControlFlow::Break(Err(3)));
    ///
    /// assert_eq!(Some(3).branch(), ControlFlow::Continue(3));
    /// assert_eq!(None::<String>.branch(), ControlFlow::Break(None));
    ///
    /// assert_eq!(ControlFlow::<String, _>::Continue(3).branch(), ControlFlow::Continue(3));
    /// assert_eq!(
    ///     ControlFlow::<_, String>::Break(3).branch(),
    ///     ControlFlow::Break(ControlFlow::Break(3)),
    /// );
    /// ```
    #[cfg_attr(not(bootstrap), lang = "branch")]
    #[unstable(feature = "try_trait_v2", issue = "84277")]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

/// Used to specify which residuals can be converted into which [`crate::ops::Try`] types.
///
/// Every `Try` type needs to be recreatable from its own associated
/// `Residual` type, but can also have additional `FromResidual` implementations
/// to support interconversion with other `Try` types.
#[rustc_on_unimplemented(
    on(
        all(
            from_method = "from_residual",
            from_desugaring = "QuestionMark",
            _Self = "std::result::Result<T, E>",
            R = "std::option::Option<std::convert::Infallible>"
        ),
        message = "the `?` operator can only be used on `Result`s, not `Option`s, \
            in {ItemContext} that returns `Result`",
        label = "use `.ok_or(...)?` to provide an error compatible with `{Self}`",
        enclosing_scope = "this function returns a `Result`"
    ),
    on(
        all(
            from_method = "from_residual",
            from_desugaring = "QuestionMark",
            _Self = "std::result::Result<T, E>",
        ),
        // There's a special error message in the trait selection code for
        // `From` in `?`, so this is not shown for result-in-result errors,
        // and thus it can be phrased more strongly than `ControlFlow`'s.
        message = "the `?` operator can only be used on `Result`s \
            in {ItemContext} that returns `Result`",
        label = "this `?` produces `{R}`, which is incompatible with `{Self}`",
        enclosing_scope = "this function returns a `Result`"
    ),
    on(
        all(
            from_method = "from_residual",
            from_desugaring = "QuestionMark",
            _Self = "std::option::Option<T>",
        ),
        // `Option`-in-`Option` always works, as there's only one possible
        // residual, so this can also be phrased strongly.
        message = "the `?` operator can only be used on `Option`s \
            in {ItemContext} that returns `Option`",
        label = "this `?` produces `{R}`, which is incompatible with `{Self}`",
        enclosing_scope = "this function returns an `Option`"
    ),
    on(
        all(
            from_method = "from_residual",
            from_desugaring = "QuestionMark",
            _Self = "std::ops::ControlFlow<B, C>",
        ),
        message = "the `?` operator can only be used on `ControlFlow<B, _>`s \
            in {ItemContext} that returns `ControlFlow<B, _>`",
        label = "this `?` produces `{R}`, which is incompatible with `{Self}`",
        enclosing_scope = "this function returns a `ControlFlow`",
        note = "unlike `Result`, there's no `From`-conversion performed for `ControlFlow`"
    ),
    on(
        all(
            from_method = "from_residual",
            from_desugaring = "QuestionMark"
        ),
        message = "the `?` operator can only be used in {ItemContext} \
                    that returns `Result` or `Option` \
                    (or another type that implements `{FromResidual}`)",
        label = "cannot use the `?` operator in {ItemContext} that returns `{Self}`",
        enclosing_scope = "this function should return `Result` or `Option` to accept `?`"
    ),
)]
#[unstable(feature = "try_trait_v2", issue = "84277")]
pub trait FromResidual<R = <Self as Try>::Residual> {
    /// Constructs the type from a compatible `Residual` type.
    ///
    /// This should be implemented consistently with the `branch` method such
    /// that applying the `?` operator will get back an equivalent residual:
    /// `FromResidual::from_residual(r).branch() --> ControlFlow::Break(r)`.
    /// (It may not be an *identical* residual when interconversion is involved.)
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(try_trait_v2)]
    /// use std::ops::{ControlFlow, FromResidual};
    ///
    /// assert_eq!(Result::<String, i64>::from_residual(Err(3_u8)), Err(3));
    /// assert_eq!(Option::<String>::from_residual(None), None);
    /// assert_eq!(
    ///     ControlFlow::<_, String>::from_residual(ControlFlow::Break(5)),
    ///     ControlFlow::Break(5),
    /// );
    /// ```
    #[cfg_attr(not(bootstrap), lang = "from_residual")]
    #[unstable(feature = "try_trait_v2", issue = "84277")]
    fn from_residual(residual: R) -> Self;
}
