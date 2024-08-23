use std::marker::PhantomData;

use polonius_the_crab::{polonius, polonius_return};
use replace_with::replace_with_or_abort;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Map<K, V>
where
    K: From<u32> + Into<u32>,
{
    data: Vec<V>,
    _idx: PhantomData<K>,
}

impl<K, V> Default for Map<K, V>
where
    K: From<u32> + Into<u32>,
{
    fn default() -> Self {
        Self {
            data: vec![],
            _idx: PhantomData,
        }
    }
}

impl<K, V> Map<K, V>
where
    K: From<u32> + Into<u32>,
{
    pub const fn new() -> Self {
        Self {
            data: vec![],
            _idx: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            _idx: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn contains(&self, k: K) -> bool {
        self.try_get(k).is_some()
    }

    pub fn as_ptr(&self) -> *const V {
        self.data.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut V {
        self.data.as_mut_ptr()
    }

    pub fn as_ref(&self) -> &Vec<V> {
        &self.data
    }

    pub fn as_mut(&mut self) -> &mut Vec<V> {
        &mut self.data
    }

    pub fn insert(&mut self, v: V) -> K {
        let idx = self.data.len();
        self.data.push(v);
        Self::idx_to_key(idx)
    }

    pub fn get(&self, k: K) -> &V {
        &self.data[Self::key_to_idx(k)]
    }

    pub unsafe fn get_unchecked(&self, k: K) -> &V {
        unsafe { self.data.get_unchecked(Self::key_to_idx(k)) }
    }

    pub fn get_mut(&mut self, k: K) -> &mut V {
        &mut self.data[Self::key_to_idx(k)]
    }

    pub unsafe fn get_unchecked_mut(&mut self, k: K) -> &mut V {
        unsafe { self.data.get_unchecked_mut(Self::key_to_idx(k)) }
    }

    pub fn try_get(&self, k: K) -> Option<&V> {
        self.data.get(Self::key_to_idx(k))
    }

    pub fn try_get_mut(&mut self, k: K) -> Option<&mut V> {
        self.data.get_mut(Self::key_to_idx(k))
    }

    pub fn get_mut_or(&mut self, default: V, k: K) -> &mut V {
        let mut this = self;
        polonius!(|this| -> &'polonius mut V {
            if let Some(v) = this.try_get_mut(k) {
                polonius_return!(v);
            }
        });
        let k = this.insert(default);
        this.get_mut(k)
    }

    pub fn get_mut_or_else<F>(&mut self, default: F, k: K) -> &mut V
    where
        F: FnOnce() -> V,
    {
        let mut this = self;
        polonius!(|this| -> &'polonius mut V {
            if let Some(v) = this.try_get_mut(k) {
                polonius_return!(v);
            }
        });
        let k = this.insert(default());
        this.get_mut(k)
    }

    pub fn get_mut_or_default(&mut self, k: K) -> &mut V
    where
        V: Default,
    {
        self.get_mut_or(V::default(), k)
    }

    pub fn set(&mut self, k: K, v: V) {
        *self.get_mut(k) = v;
    }

    pub fn set_with<F>(&mut self, k: K, f: F)
    where
        F: FnOnce(V) -> V,
    {
        replace_with_or_abort(self.get_mut(k), f)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (K, &'a V)> {
        self.data
            .iter()
            .enumerate()
            .map(|(idx, v)| (Self::idx_to_key(idx), v))
    }

    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.data
            .iter()
            .enumerate()
            .map(|(idx, _)| Self::idx_to_key(idx))
    }

    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a V> {
        self.data.iter()
    }

    fn idx_to_key(idx: usize) -> K {
        K::from(idx as u32)
    }

    fn key_to_idx(k: K) -> usize {
        let raw: u32 = k.into();
        raw as usize
    }
}
