// use std::{marker::PhantomData, num::NonZeroU32};

// pub struct Idx<T> {
//     raw: NonZeroU32,
//     _ty: PhantomData<T>,
// }

// impl<T> Idx<T> {
//     pub fn from_idx(idx: u32) -> Self {
//         Self {
//             raw: Self::idx_to_raw(idx),
//             _ty: PhantomData,
//         }
//     }

//     pub fn into_raw(&self) -> u32 {
//         self.raw.into()
//     }

//     pub fn into_usize(&self) -> usize {
//         self.into_raw() as usize
//     }

//     fn idx_to_raw(idx: u32) -> NonZeroU32 {
//         match idx {
//             0..u32::MAX => unsafe { NonZeroU32::new_unchecked(idx + 1) },
//             u32::MAX => panic!("cannot convert `u32::MAX` into `Idx`"),
//         }
//     }
// }

use std::ops::{Deref, DerefMut};

#[macro_export]
macro_rules! nz_ids {
	($($name:ident),*) => {
		paste::paste! {
			$(
				#[repr(transparent)]
				#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
				pub struct $name(std::num::NonZeroU32);

				impl [<$name>] {
						pub fn new(value: u32) -> Self {
								Self(std::num::NonZeroU32::new(value).unwrap_or_else(|| flux_diagnostics::ice(format!("invalid `{}Id`: {value}", stringify!($name)))))
						}

						pub const unsafe fn new_unchecked(value: u32) -> Self {
								Self(std::num::NonZeroU32::new_unchecked(value))
						}

						pub fn raw(&self) -> u32 {
								self.0.into()
						}

						pub const fn from_idx(value: u32) -> Self {
							match value {
								0..u32::MAX => unsafe { Self::new_unchecked(value + 1) },
								u32::MAX => panic!(),
					 }
						}
				}

				impl From<u32> for $name {
						fn from(value: u32) -> Self {
								match value {
										 0..u32::MAX => unsafe { $name::new_unchecked(value + 1) },
											u32::MAX => panic!("cannot convert `usize::MAX` into `{}Id`", stringify!($name)),
								}
						}
				}

				impl Into<u32> for $name {
						fn into(self) -> u32 {
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
				pub struct $name(u32);

				impl $name {
						pub const fn new(id: u32) -> Self {
								Self(id)
						}

						pub fn raw(&self) -> u32 {
								self.0
						}
				}

				impl From<u32> for $name {
						fn from(value: u32) -> Self {
								Self(value)
						}
				}

				impl Into<u32> for $name {
						fn into(self) -> u32 {
								self.raw()
						}
				}
			)*
		}
	};
}

nz_ids!(TraitDecl, ApplyDecl);
nz_ids!(Ty, Mod, Expr, Pkg, EnumDecl, FnDecl, ModDecl, StructDecl, UseDecl);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InPkg<T> {
    pub pkg_id: Pkg,
    pub inner: T,
}

impl<T> Copy for InPkg<T> where T: Copy {}

impl<T> Deref for InPkg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for InPkg<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait WithPackage {
    fn in_pkg(self, pkg_id: Pkg) -> InPkg<Self>
    where
        Self: Sized;
}

impl<T> WithPackage for T
where
    T: Sized,
{
    fn in_pkg(self, pkg_id: Pkg) -> InPkg<Self> {
        InPkg {
            pkg_id,
            inner: self,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InMod<T> {
    pub mod_id: Mod,
    pub inner: T,
}

impl<T> Copy for InMod<T> where T: Copy {}

impl<T> Deref for InMod<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for InMod<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait WithMod {
    fn in_mod(self, mod_id: Mod) -> InMod<Self>
    where
        Self: Sized;
}

impl<T> WithMod for T
where
    T: Sized,
{
    fn in_mod(self, mod_id: Mod) -> InMod<Self> {
        InMod {
            mod_id,
            inner: self,
        }
    }
}
