use crate::test::TestArgs;
use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

mod test;

/// Wraps a database-backed async test in the temporary DB helper and, optionally,
/// a task-local context scope.
///
/// Usage:
/// ```rust,no_run,ignore
/// use crate::{db::DbHandle, state::TaskContext};
///
/// #[biois::test(task_context = TaskContext::new(user))]
/// async fn it_tests_something(db: DbHandle) {
///     let _ = db;
/// }
/// ```
#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as TestArgs);
    let input = parse_macro_input!(item as ItemFn);
    let expanded = crate::test::test(&args, &input);
    TokenStream::from(expanded)
}
