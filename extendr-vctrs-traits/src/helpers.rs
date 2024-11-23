use extendr_api::prelude::*;

// Helper functions for creating your own vectors
// Takes a Vec<Option<T>> and captures the debug output as a character vec
pub fn vctr_show<T: std::fmt::Debug, S: AsRef<[Option<T>]>>(x: S) -> Strings {
    x.as_ref()
        .iter()
        .map(|xi| match xi {
            Some(xi) => format!("{:?}", xi),
            None => String::from("NA"),
        })
        .collect::<Strings>()
}

// Returns an integer of the length of the vector
pub fn vctr_len<T: std::fmt::Debug, S: AsRef<[Option<T>]>>(x: S) -> Rint {
    Rint::from(x.as_ref().len() as i32)
}

// extracts elements of a vector with copying unfortunately
pub fn vctr_subset<T: Clone>(x: Vec<Option<T>>, idx: Integers) -> Vec<Option<T>> {
    let x_len = x.len();

    let res: Vec<_> = idx
        .into_iter()
        .map(|i| match i {
            _ if i.is_na() => None::<T>,
            _ if i.inner() <= 0 || i.inner() as usize > x_len => None,
            _ => x.get(i.inner() as usize - 1).cloned().unwrap_or(None),
        })
        .collect();
    // x[(i.inner() as usize) - 1]
    res
}

// this function takes two vectors and combines them into 1
pub fn vctr_extend<T>(mut x: Vec<Option<T>>, y: Vec<Option<T>>) -> Vec<Option<T>> {
    x.extend(y.into_iter());
    x
}
