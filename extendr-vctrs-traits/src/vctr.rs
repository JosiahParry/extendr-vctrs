use crate::rvctr::Rvctr;
use extendr_api::prelude::*;

#[derive(Debug, Clone)]
pub struct Vctr<T: Rvctr> {
    pub(crate) inner: Integers,
    phantom: std::marker::PhantomData<T>,
}

/// Convert from an Robj to a Vctr container
impl<T: Rvctr> TryFrom<Robj> for Vctr<T> {
    type Error = extendr_api::Error;

    fn try_from(value: Robj) -> Result<Self> {
        let inner = Integers::try_from(value)?;

        // Check that the point is an external pointer
        let ptr = match inner.get_attrib("extendr_ptr") {
            Some(ptr) => ptr,
            None => return Err(Error::ExpectedExternalPtr(().into())),
        };

        // Here we try to convert to the external pointer
        // if this fails it is the wrong type
        let _ = ExternalPtr::<T>::try_from(&ptr)?;

        // craft the vector from the integer
        let res = Vctr {
            inner,
            phantom: std::marker::PhantomData,
        };

        Ok(res)
    }
}

impl<T: Rvctr> Vctr<T> {
    pub fn as_vctr(&self) -> Robj {
        let mut x = self.inner.clone();
        let t_class = T::class();
        if t_class == "extendr_vctr" {
            x.set_class([t_class, "vctrs_vctr"])
                .expect("failed to set class")
        } else {
            x.set_class([T::class(), "extendr_vctr", "vctrs_vctr"])
                .expect("failed to set class")
        };

        x.into_robj()
    }

    pub fn try_into_inner(&self) -> Result<ExternalPtr<T>> {
        // Extract the "extendr_ptr" attribute
        let ptr_attrib = self
            .inner
            .get_attrib("extendr_ptr")
            .ok_or_else(|| extendr_api::Error::ExpectedExternalPtr(().into()))?;

        // Convert the attribute into an external pointer
        let external_ptr = ExternalPtr::<T>::try_from(&ptr_attrib)?;
        Ok(external_ptr)
    }

    pub fn show(&self) -> Result<Strings> {
        let inner = self.try_into_inner()?;
        Ok(inner.show())
    }
}

/// Convert from `T` to a Vctr container
impl<T: Rvctr> From<T> for Vctr<T> {
    fn from(value: T) -> Self {
        let n = value.length();
        let ptr = ExternalPtr::new(value);
        let mut inner = Integers::new(n.inner() as usize);
        let inner = inner.set_attrib("extendr_ptr", ptr).unwrap().clone();
        Vctr {
            inner,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Rvctr> From<Vctr<T>> for Robj {
    fn from(value: Vctr<T>) -> Self {
        value.as_vctr()
    }
}
