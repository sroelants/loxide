use std::hash::Hash;
use std::fmt::Debug;

pub struct RefEq<'a, T>(pub &'a T);

impl<'a, T> Debug for RefEq<'a, T> where T: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RefEq")
         .field(&format!("{:p}", self.0))
         .field(&self.0)
         .finish()
    }
}

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
