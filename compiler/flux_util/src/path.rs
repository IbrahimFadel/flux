use std::convert::Infallible;

use crate::{Interner, Word};

pub trait PathSegment {
    fn as_str(&self, interner: &'static Interner) -> &str;
    fn to_string(&self, interner: &'static Interner) -> String;
}

impl PathSegment for Word {
    fn as_str(&self, interner: &'static Interner) -> &str {
        interner.resolve(self)
    }

    fn to_string(&self, interner: &'static Interner) -> String {
        self.as_str(interner).to_string()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Path<T = Word, A = Infallible>
where
    T: PathSegment,
{
    pub segments: Vec<T>,
    pub args: Vec<A>,
}

impl<T, A> Path<T, A>
where
    T: PathSegment,
{
    pub fn new(segments: Vec<T>, args: Vec<A>) -> Self {
        Self { segments, args }
    }

    pub fn empty() -> Self {
        Self {
            segments: vec![],
            args: vec![],
        }
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.segments.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.segments.into_iter()
    }

    pub fn try_get_nth(&self, n: usize) -> Option<&T> {
        self.segments.get(n)
    }

    pub fn get_nth(&self, n: usize) -> &T {
        &self.segments[n]
    }

    pub fn last(&self) -> Option<&T> {
        self.segments.last()
    }

    pub fn to_string(&self, interner: &'static Interner) -> String {
        if self.len() == 0 {
            String::from("This")
        } else {
            self.iter()
                .map(|seg| seg.as_str(interner))
                .collect::<Vec<&str>>()
                .join("::")
        }
    }

    pub fn discard_args(self) -> Path<T, Infallible> {
        Path {
            segments: self.segments,
            args: vec![],
        }
    }

    pub fn is_in<const N: usize>(&self, paths: &[Self; N]) -> bool
    where
        T: PartialEq,
    {
        paths
            .iter()
            .find(|path| path.segments == self.segments)
            .is_some()
    }

    pub fn map_args<B, F>(self, mut with: F) -> Path<T, B>
    where
        F: FnMut(A) -> B,
    {
        Path {
            segments: self.segments,
            args: self.args.into_iter().map(|arg| with(arg)).collect(),
        }
    }
}

impl<T> Path<T, Infallible>
where
    T: PathSegment,
{
    pub fn allow_args<A>(self) -> Path<T, A> {
        Path {
            segments: self.segments,
            args: vec![],
        }
    }
}

// pub trait Pretty {
//     fn to_doc(&self, interner: &'static Interner) -> RcDoc;
// }

// impl<T: PathSegment, A: Pretty> Path<T, A> {
//     pub fn to_doc<'a>(&'a self, interner: &'static Interner) -> RcDoc {
//         let segments = RcDoc::intersperse(
//             self.iter()
//                 .map(|segment| RcDoc::text(segment.as_str(interner).yellow().to_string())),
//             RcDoc::text("::".black().to_string()),
//         );
//         let generics = if self.args.is_empty() {
//             RcDoc::nil()
//         } else {
//             let generics = RcDoc::intersperse(
//                 self.args.iter().map(|ty| ty.to_doc(interner)),
//                 RcDoc::text(",").append(RcDoc::space()),
//             );
//             RcDoc::text("<".black().to_string())
//                 .append(generics)
//                 .append(RcDoc::text(">".black().to_string()))
//         };
//         segments.append(generics)
//     }
// }

// impl Pretty for Infallible {
//     fn to_doc(&self, _: &'static Interner) -> RcDoc {
//         RcDoc::nil()
//     }
// }
