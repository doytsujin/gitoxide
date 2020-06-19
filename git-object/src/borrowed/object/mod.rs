mod error;
pub use error::Error;

use crate::Time;
use bstr::BStr;

use crate::borrowed::{Commit, Tag, Tree};

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Signature<'data> {
    pub name: &'data BStr,
    pub email: &'data BStr,
    pub time: Time,
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum Object<'data> {
    Tag(Tag<'data>),
    Commit(Commit<'data>),
    Tree(Tree<'data>),
}

impl<'data> Object<'data> {
    pub fn kind(&self) -> crate::Kind {
        match self {
            Object::Tag(_) => crate::Kind::Tag,
            Object::Commit(_) => crate::Kind::Commit,
            Object::Tree(_) => crate::Kind::Tree,
        }
    }
}

mod convert {
    use crate::borrowed::{Commit, Object, Tag, Tree};
    use std::convert::TryFrom;

    impl<'data> From<Tag<'data>> for Object<'data> {
        fn from(v: Tag<'data>) -> Self {
            Object::Tag(v)
        }
    }

    impl<'data> From<Commit<'data>> for Object<'data> {
        fn from(v: Commit<'data>) -> Self {
            Object::Commit(v)
        }
    }

    impl<'data> From<Tree<'data>> for Object<'data> {
        fn from(v: Tree<'data>) -> Self {
            Object::Tree(v)
        }
    }

    impl<'data> TryFrom<Object<'data>> for Tag<'data> {
        type Error = Object<'data>;

        fn try_from(value: Object<'data>) -> Result<Self, Self::Error> {
            Ok(match value {
                Object::Tag(v) => v,
                _ => return Err(value),
            })
        }
    }

    impl<'data> TryFrom<Object<'data>> for Commit<'data> {
        type Error = Object<'data>;

        fn try_from(value: Object<'data>) -> Result<Self, Self::Error> {
            Ok(match value {
                Object::Commit(v) => v,
                _ => return Err(value),
            })
        }
    }

    impl<'data> TryFrom<Object<'data>> for Tree<'data> {
        type Error = Object<'data>;

        fn try_from(value: Object<'data>) -> Result<Self, Self::Error> {
            Ok(match value {
                Object::Tree(v) => v,
                _ => return Err(value),
            })
        }
    }
}