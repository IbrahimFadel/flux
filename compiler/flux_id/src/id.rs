use std::{
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
};

use flux_diagnostics::ice;

macro_rules! nz_ids {
	($($name:ident),*) => {
		paste::paste! {
			$(
				#[repr(transparent)]
				#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
				pub struct $name(NonZeroUsize);

				impl [<$name>] {
						pub fn new(value: usize) -> Self {
								Self(NonZeroUsize::new(value).unwrap_or_else(|| ice(format!("invalid `{}Id`: {value}", stringify!($name)))))
						}

						pub unsafe fn new_unchecked(value: usize) -> Self {
								Self(NonZeroUsize::new_unchecked(value))
						}

						pub fn raw(&self) -> usize {
								self.0.into()
						}
				}

				impl From<usize> for $name {
						fn from(value: usize) -> Self {
								match value {
										usize::MAX => panic!("cannot convert `usize::MAX` into `{}Id`", stringify!($name)),
										_ => unsafe { $name::new_unchecked(value + 1) },
								}
						}
				}

				impl Into<usize> for $name {
						fn into(self) -> usize {
								match self.raw() {
										0 => unreachable!(),
										v => v - 1,
								}
						}
				}
			)*
		}
	};
}

#[macro_export]
macro_rules! ids {
	($($name:ident),*) => {
		paste::paste! {
			$(
				#[repr(transparent)]
				#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
				pub struct $name(usize);

				impl $name {
						pub const fn new(id: usize) -> Self {
								Self(id)
						}

						pub fn raw(&self) -> usize {
								self.0
						}
				}

				impl From<usize> for $name {
						fn from(value: usize) -> Self {
								Self(value)
						}
				}

				impl Into<usize> for $name {
						fn into(self) -> usize {
								self.raw()
						}
				}
			)*
		}
	};
}

nz_ids!(TraitDecl, ApplyDecl);
ids!(Ty, Mod, Expr, Pkg, EnumDecl, FnDecl, ModDecl, StructDecl, UseDecl);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct P<T> {
    pub pkg_id: Pkg,
    pub inner: T,
}

impl<T> Copy for P<T> where T: Copy {}

impl<T> Deref for P<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for P<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait InPkg {
    fn in_pkg(self, pkg_id: Pkg) -> P<Self>
    where
        Self: Sized;
}

impl<T> InPkg for T
where
    T: Sized,
{
    fn in_pkg(self, pkg_id: Pkg) -> P<Self> {
        P {
            pkg_id,
            inner: self,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct M<T> {
    pub mod_id: Mod,
    pub inner: T,
}

impl<T> Copy for M<T> where T: Copy {}

impl<T> Deref for M<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for M<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait InMod {
    fn in_mod(self, mod_id: Mod) -> M<Self>
    where
        Self: Sized;
}

impl<T> InMod for T
where
    T: Sized,
{
    fn in_mod(self, mod_id: Mod) -> M<Self> {
        M {
            mod_id,
            inner: self,
        }
    }
}
