/// A collection of errors, from zero to multiple.
#[derive(Default)]
pub(crate) struct SynErrors {
    errors: Option<syn::Error>,
}

impl SynErrors {
    pub(crate) fn push(&mut self, new_error: syn::Error) {
        match &mut self.errors {
            Some(errors) => errors.combine(new_error),
            None => self.errors = Some(new_error),
        }
    }

    /// Returns [`Err`] if there are errors, or [`Ok`] if there are none.
    ///
    /// Intended to use with the question mark operator:
    /// ```ignore
    /// errors.into_result()?;
    /// ```
    pub(crate) fn into_result(self) -> syn::Result<()> {
        match self.errors {
            Some(errors) => Err(errors),
            None => Ok(()),
        }
    }
}

/// Imprecise because one can shadow the primitives, but let's hope nobody does that :)
pub(crate) fn is_unsigned_int_primitive(ty: &syn::Type) -> bool {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = ty {
        if let Some(base_ty) = path.get_ident() {
            if ["u8", "u16", "u32", "u64", "u128", "usize"].contains(&&*base_ty.to_string()) {
                return true;
            }
        }
    }
    false
}
