use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Error as SynError, FnArg, ItemFn, parse::Parse};

pub fn test(args: &TestArgs, input: &ItemFn) -> TokenStream {
    let fn_attrs = input.attrs.iter().filter(|attr| !is_test_attr(attr));
    let vis = &input.vis;
    let asyncness = &input.sig.asyncness;
    let ident = &input.sig.ident;
    let generics = &input.sig.generics;
    let output = &input.sig.output;
    let stmts = &input.block.stmts;

    let inputs = &input.sig.inputs;
    let mut iter = inputs.iter();
    let first_param = iter.next();
    let trailing_params = iter.collect::<Vec<_>>();

    let first_param = match first_param {
        Some(FnArg::Typed(pat_type)) => pat_type,
        Some(FnArg::Receiver(_)) => {
            return SynError::new_spanned(
                ident,
                "crate::test does not support methods with `self` receivers",
            )
            .to_compile_error();
        }
        None => {
            return SynError::new_spanned(
                ident,
                "crate::test requires a first parameter like `db: DbHandle`",
            )
            .to_compile_error();
        }
    };

    if !trailing_params.is_empty() {
        return SynError::new_spanned(
            &first_param.ty,
            "crate::test currently supports only a single `db: DbHandle` parameter",
        )
        .to_compile_error();
    }

    let db_binding = &first_param.pat;
    let task_context = args.task_context.as_ref();

    let scoped_body = if let Some(task_context) = task_context {
        quote! {
            crate::state::CONTEXT
                .scope(#task_context, async {
                    #(#stmts)*
                })
                .await
        }
    } else {
        quote! {
            #(#stmts)*
        }
    };

    quote! {
        #(#fn_attrs)*
        #[tokio::test(flavor = "multi_thread")]
        #vis #asyncness fn #ident #generics () #output {
            crate::db::tests::with_temp_db(|#db_binding| async move {
                #scoped_body
            })
            .await;
        }
    }
}

pub struct TestArgs {
    task_context: Option<syn::Expr>,
}

impl Parse for TestArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let mut task_context = None;

        while !input.is_empty() {
            let name: syn::Ident = input.parse()?;
            let _eq: syn::token::Eq = input.parse()?;
            let value: syn::Expr = input.parse()?;

            if name == "task_context" {
                task_context = Some(value);
            } else {
                return Err(SynError::new_spanned(
                    name,
                    "unsupported attribute argument; expected `task_context = ...`",
                ));
            }

            if input.is_empty() {
                break;
            }
            let _comma: syn::token::Comma = input.parse()?;
        }

        Ok(Self { task_context })
    }
}

fn is_test_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("test") || attr.path().is_ident("tokio")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_generates_expected_token_output_for_a_db_test() {
        let input: ItemFn = syn::parse_quote! {
            async fn it_tests_something(db: crate::db::DbHandle) {
                let value = db.schema_name();
                assert!(!value.is_empty());
            }
        };
        let args: TestArgs = syn::parse_quote!(task_context = crate::state::TaskContext::new(user));

        let actual = test(args, input).to_string();
        let expected = quote! {
            #[tokio::test(flavor = "multi_thread")]
            async fn it_tests_something() {
                crate::db::tests::with_temp_db(|db| async move {
                    crate::state::CONTEXT
                        .scope(crate::state::TaskContext::new(user), async {
                            let value = db.schema_name();
                            assert!(!value.is_empty());
                        })
                        .await
                })
                .await;
            }
        }
        .to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_generates_expected_token_output_without_task_context() {
        let input: ItemFn = syn::parse_quote! {
            async fn it_tests_something(db: crate::db::DbHandle) {
                let value = db.schema_name();
                assert!(!value.is_empty());
            }
        };
        let args: TestArgs = syn::parse_quote!();

        let actual = test(args, input).to_string();
        let expected = quote! {
            #[tokio::test(flavor = "multi_thread")]
            async fn it_tests_something() {
                crate::db::tests::with_temp_db(|db| async move {
                    let value = db.schema_name();
                    assert!(!value.is_empty());
                })
                .await;
            }
        }
        .to_string();

        assert_eq!(actual, expected);
    }
}
