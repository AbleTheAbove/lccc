/// Enum Representing Control Flow for the [`try!`] macro
pub enum ControlFlow<B, C> {
    /// Continue execution
    Continue(C),
    /// Break execution
    Break(B),
}

/// Allows construction of types from Residuals
pub trait FromResidual<Residual = <Self as Try>::Residual> {
    /// Constructs from a residual value
    fn from_residual(res: Residual) -> Self;
}

/// Reimplementation of the standard [`Try`] trait.
pub trait Try: FromResidual {
    /// The residual of the type
    type Residual;
    /// The Output of the type
    type Output;

    /// Converts the `Output` type into type.
    fn from_output(output: Self::Output) -> Self;

    /// Branches into a Residual or Output value
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

/// An Empty enum, used for residuals
pub enum Empty {}

impl<T, E> FromResidual for crate::result::Result<T, E> {
    fn from_residual(res: crate::result::Result<Empty, E>) -> Self {
        match res {
            crate::result::Result::Ok(v) => match v {},
            crate::result::Result::Err(e) => Self::Err(e),
        }
    }
}

impl<T, E> FromResidual<Result<core::convert::Infallible, E>> for crate::result::Result<T, E> {
    fn from_residual(res: Result<core::convert::Infallible, E>) -> Self {
        match res {
            Ok(v) => match v {},
            Err(e) => Self::Err(e),
        }
    }
}

impl<T, E> Try for crate::result::Result<T, E> {
    type Residual = crate::result::Result<Empty, E>;
    type Output = T;

    fn from_output(output: T) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(c) => ControlFlow::Continue(c),
            Self::Err(e) => ControlFlow::Break(crate::result::Result::Err(e)),
        }
    }
}

impl<T, E> FromResidual for Result<T, E> {
    // use `core:;convert::Infallible` instead of `Empty`
    fn from_residual(x: Result<core::convert::Infallible, E>) -> Self {
        match x {
            Ok(v) => match v {},
            Err(e) => Err(e),
        }
    }
}

impl<T, E> FromResidual<crate::result::Result<Empty, E>> for Result<T, E> {
    // use `core:;convert::Infallible` instead of `Empty`
    fn from_residual(x: crate::result::Result<Empty, E>) -> Self {
        match x {
            crate::result::Ok(v) => match v {},
            crate::result::Err(e) => Err(e),
        }
    }
}

impl<T, E> Try for Result<T, E> {
    type Residual = Result<core::convert::Infallible, E>;
    type Output = T;

    fn from_output(output: Self::Output) -> Self {
        Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(val) => ControlFlow::Continue(val),
            Err(e) => ControlFlow::Break(Err(e)),
        }
    }
}

/// `?` operator for `xlang_abi` types
#[macro_export]
macro_rules! try_ {
    ($expr:expr) => {{
        match $crate::ops::Try::branch($expr) {
            $crate::ops::ControlFlow::Continue(val) => val,
            $crate::ops::ControlFlow::Break(residual) => {
                return $crate::ops::FromResidual::from_residual(residual)
            }
        }
    }};
}

#[cfg(test)]
mod test {
    use crate::prelude::v1::*;

    #[test]
    fn test_control_flow() -> std::result::Result<(), ()> {
        let res = crate::result::Result::Ok(1);
        let val = try_!(res);
        assert_eq!(val, 1);
        Ok(())
    }
}
