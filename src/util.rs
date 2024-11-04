use std::hash::Hash;

pub struct RefEq<'a, T>(pub &'a T);

impl<'a, T> Hash for RefEq<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const T).hash(state);
    }
}

impl<'a, T> PartialEq for RefEq<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const T == other.0 as *const T
    }
}

impl<'a, T> Eq for RefEq<'a, T> {}
