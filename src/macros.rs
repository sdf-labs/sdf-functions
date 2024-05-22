
macro_rules! export_functions {
    ($(($DIALECT:ident, $FUNC:ident,  $($arg:ident)*, $DOC:expr)),*) => {
        pub mod expr_fn {
            $(
                #[doc = $DOC]
                /// Return $name(arg)
                pub fn $FUNC($($arg: datafusion::logical_expr::Expr),*) -> datafusion::logical_expr::Expr {
                    super::$FUNC().call(vec![$($arg),*],)
                }
            )*
        }

        /// Return a list of all functions in this package
        pub fn functions() -> Vec<(String, std::sync::Arc<datafusion::logical_expr::ScalarUDF>)> {
            
            vec![
                $(
                    (format!("{}::{}", stringify!($DIALECT), stringify!($FUNC)), $FUNC()),
                )*
            ]
            
        }
    };
}

/// Creates a singleton `ScalarUDF` of the `$UDF` function named `$GNAME` and a
/// function named `$NAME` which returns that function named $NAME.
///
/// This is used to ensure creating the list of `ScalarUDF` only happens once.
macro_rules! make_udf_function {
    ($UDF:ty, $GNAME:ident, $NAME:ident) => {
        /// Singleton instance of the function
        static $GNAME: std::sync::OnceLock<std::sync::Arc<datafusion::logical_expr::ScalarUDF>> =
            std::sync::OnceLock::new();

        /// Return a [`ScalarUDF`] for [`$UDF`]
        ///
        /// [`ScalarUDF`]: datafusion::logical_expr::ScalarUDF
        fn $NAME() -> std::sync::Arc<datafusion::logical_expr::ScalarUDF> {
            $GNAME
                .get_or_init(|| {
                    std::sync::Arc::new(datafusion::logical_expr::ScalarUDF::new_from_impl(
                        <$UDF>::new(),
                    ))
                })
                .clone()
        }
    };
}

/// Macro creates the named module if the feature is enabled
/// otherwise creates a stub
///
/// Which returns:
///
/// 1. The list of actual function implementation when the relevant
/// feature is activated,
///
/// 2. A list of stub function when the feature is not activated that produce
/// a runtime error (and explain what feature flag is needed to activate them).
///
/// The rationale for providing stub functions is to help users to configure datafusion
/// properly (so they get an error telling them why a function is not available)
/// instead of getting a cryptic "no function found" message at runtime.

macro_rules! make_package {
    ($name:ident, $feature:literal, $DOC:expr) => {
        #[cfg(feature = $feature)]
        #[doc = $DOC ]
        #[doc = concat!("Enabled via feature flag `", $feature, "`")]
        pub mod $name;

        #[cfg(not(feature = $feature))]
        #[doc = concat!("Disabled. Enable via feature flag `", $feature, "`")]
        pub mod $name {
            use datafusion::logical_expr::ScalarUDF;
            use log::debug;
            use std::sync::Arc;

            /// Returns an empty list of functions when the feature is not enabled
            pub fn functions() -> Vec<Arc<ScalarUDF>> {
                debug!("{} functions disabled", stringify!($name));
                vec![]
            }
        }
    };
}

/// Invokes a function on each element of an array and returns the result as a new array
///
/// $ARG: ArrayRef
/// $NAME: name of the function (for error messages)
/// $ARGS_TYPE: the type of array to cast the argument to
/// $RETURN_TYPE: the type of array to return
/// $FUNC: the function to apply to each element of $ARG
///
macro_rules! make_function_scalar_inputs_return_type {
    ($ARG: expr, $NAME:expr, $ARG_TYPE:ident, $RETURN_TYPE:ident, $FUNC: block) => {{
        let arg = downcast_arg!($ARG, $NAME, $ARG_TYPE);

        arg.iter()
            .map(|a| match a {
                Some(a) => Some($FUNC(a)),
                _ => None,
            })
            .collect::<$RETURN_TYPE>()
    }};
}

/// Downcast an argument to a specific array type, returning an internal error
/// if the cast fails
///
/// $ARG: ArrayRef
/// $NAME: name of the argument (for error messages)
/// $ARRAY_TYPE: the type of array to cast the argument to
macro_rules! downcast_arg {
    ($ARG:expr, $NAME:expr, $ARRAY_TYPE:ident) => {{
        $ARG.as_any().downcast_ref::<$ARRAY_TYPE>().ok_or_else(|| {
            DataFusionError::Internal(format!(
                "could not cast {} to {}",
                $NAME,
                std::any::type_name::<$ARRAY_TYPE>()
            ))
        })?
    }};
}