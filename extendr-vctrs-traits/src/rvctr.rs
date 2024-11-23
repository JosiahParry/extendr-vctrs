use extendr_api::prelude::*;

// This is the trait that all Rust objects that want to
// be treated as R vectors will need to implement
pub trait Rvctr
where
    Self: Sized + std::fmt::Debug,
{
    fn class() -> &'static str;
    fn show(&self) -> Strings;
    fn length(&self) -> Rint;
    fn subset(self, idx: Integers) -> Self;
    fn extend(self, y: Self) -> Self;
}

impl<T: std::fmt::Debug + Clone> Rvctr for Vec<Option<T>> {
    fn class() -> &'static str {
        "extendr_vctr"
    }

    fn show(&self) -> Strings {
        crate::helpers::vctr_show(&self)
    }

    fn length(&self) -> Rint {
        crate::helpers::vctr_len(&self)
    }

    fn subset(self, idx: Integers) -> Self {
        crate::helpers::vctr_subset(self, idx)
    }

    fn extend(self, y: Self) -> Self {
        crate::helpers::vctr_extend(self, y)
    }
}
