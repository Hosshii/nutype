mod error;
mod traits;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Generics};

use crate::common::{
    gen::{
        error::gen_error_type_name, tests::gen_test_should_have_valid_default_value,
        traits::GeneratedTraits, GenerateNewtype,
    },
    models::{ErrorTypeName, Guard, TypeName, TypedCustomFunction},
};

use self::error::gen_validation_error_type;

use super::{
    models::{AnyDeriveTrait, AnyGuard, AnyInnerType, AnySanitizer, AnyValidator},
    AnyNewtype,
};

use traits::gen_traits;

impl GenerateNewtype for AnyNewtype {
    type Sanitizer = AnySanitizer;
    type Validator = AnyValidator;
    type InnerType = AnyInnerType;
    type TypedTrait = AnyDeriveTrait;

    fn gen_fn_sanitize(
        inner_type: &Self::InnerType,
        sanitizers: &[Self::Sanitizer],
    ) -> TokenStream {
        let transformations: TokenStream = sanitizers
            .iter()
            .map(|san| match san {
                AnySanitizer::With(custom_sanitizer) => {
                    let inner_type_ref: syn::Type = parse_quote!(
                        #inner_type
                    );
                    let typed_sanitizer: TypedCustomFunction = custom_sanitizer
                        .clone()
                        .try_into_typed(&inner_type_ref)
                        .expect("Failed to convert `with` sanitizer into a typed closure");
                    quote!(
                        value = (#typed_sanitizer)(value);
                    )
                }
            })
            .collect();

        quote!(
            fn __sanitize__(mut value: #inner_type) -> #inner_type {
                #transformations
                value
            }
        )
    }

    fn gen_fn_validate(
        inner_type: &Self::InnerType,
        type_name: &TypeName,
        validators: &[Self::Validator],
    ) -> TokenStream {
        let error_name = gen_error_type_name(type_name);

        let validations: TokenStream = validators
            .iter()
            .map(|validator| match validator {
                AnyValidator::Predicate(predicate) => {
                    let inner_type_ref: syn::Type = parse_quote!(
                        &'nutype_a #inner_type
                    );
                    let typed_predicate: TypedCustomFunction = predicate
                        .clone()
                        .try_into_typed(&inner_type_ref)
                        .expect("Failed to convert predicate into a typed closure");
                    quote!(
                        if !(#typed_predicate)(val) {
                            return Err(#error_name::PredicateViolated);
                        }
                    )
                }
            })
            .collect();

        quote!(
            // NOTE 1: we're using a unique lifetime name `nutype_a` in a hope that it will not clash
            // with any other lifetimes in the user's code.
            //
            // NOTE 2:
            // When inner type is Cow<'a, str>, the generated code will look like this (with 2
            // lifetimes):
            //
            //     fn __validate__<'nutype_a>(val: &'nutype_a Cow<'a, str>)
            //
            // Clippy does not like passing a reference to a Cow. So we need to ignore the `clippy::ptr_arg` warning.
            // Since this code is generic which is used for different inner types (not only Cow), we cannot easily fix it to make
            // clippy happy.
            #[allow(clippy::ptr_arg)]
            fn __validate__<'nutype_a>(val: &'nutype_a #inner_type) -> ::core::result::Result<(), #error_name> {
                #validations
                Ok(())
            }
        )
    }

    fn gen_validation_error_type(
        type_name: &TypeName,
        validators: &[Self::Validator],
    ) -> TokenStream {
        gen_validation_error_type(type_name, validators)
    }

    fn gen_traits(
        type_name: &TypeName,
        generics: &Generics,
        inner_type: &Self::InnerType,
        maybe_error_type_name: Option<ErrorTypeName>,
        traits: HashSet<Self::TypedTrait>,
        maybe_default_value: Option<syn::Expr>,
        guard: &AnyGuard,
    ) -> Result<GeneratedTraits, syn::Error> {
        gen_traits(
            type_name,
            generics,
            inner_type,
            maybe_error_type_name,
            traits,
            maybe_default_value,
            guard,
        )
    }

    fn gen_tests(
        type_name: &TypeName,
        _inner_type: &Self::InnerType,
        maybe_default_value: &Option<syn::Expr>,
        guard: &Guard<Self::Sanitizer, Self::Validator>,
        _traits: &HashSet<Self::TypedTrait>,
    ) -> TokenStream {
        let test_valid_default_value = gen_test_should_have_valid_default_value(
            type_name,
            maybe_default_value,
            guard.has_validation(),
        );

        quote! {
            #test_valid_default_value
        }
    }
}
