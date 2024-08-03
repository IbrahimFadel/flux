use std::marker::PhantomData;

use replace_with::replace_with_or_abort;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Map<K, V> {
    data: Vec<V>,
    _idx: PhantomData<K>,
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self {
            data: vec![],
            _idx: PhantomData,
        }
    }
}

impl<K, V> Map<K, V> {
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

    pub fn insert(&mut self, v: V) -> K
    where
        K: From<usize>,
    {
        let idx = self.data.len();
        self.data.push(v);
        K::from(idx)
    }

    pub fn get(&self, k: K) -> &V
    where
        K: Into<usize>,
    {
        &self.data[k.into()]
    }

    pub unsafe fn get_unchecked(&self, k: K) -> &V
    where
        K: Into<usize>,
    {
        unsafe { self.data.get_unchecked(k.into()) }
    }

    pub fn get_mut(&mut self, k: K) -> &mut V
    where
        K: Into<usize>,
    {
        &mut self.data[k.into()]
    }

    pub unsafe fn get_unchecked_mut(&mut self, k: K) -> &mut V
    where
        K: Into<usize>,
    {
        unsafe { self.data.get_unchecked_mut(k.into()) }
    }

    pub fn try_get(&self, k: K) -> Option<&V>
    where
        K: Into<usize>,
    {
        self.data.get(k.into())
    }

    pub fn try_get_mut(&mut self, k: K) -> Option<&mut V>
    where
        K: Into<usize>,
    {
        self.data.get_mut(k.into())
    }

    pub fn set(&mut self, k: K, v: V)
    where
        K: Into<usize>,
    {
        *self.get_mut(k) = v;
    }

    pub fn set_with<F>(&mut self, k: K, f: F)
    where
        K: Into<usize>,
        F: FnOnce(V) -> V,
    {
        replace_with_or_abort(self.get_mut(k), f)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (K, &'a V)>
    where
        K: From<usize>,
    {
        self.data
            .iter()
            .enumerate()
            .map(|(idx, v)| (K::from(idx), v))
    }

    pub fn keys(&self) -> impl Iterator<Item = K> + '_
    where
        K: From<usize>,
    {
        self.data.iter().enumerate().map(|(idx, _)| K::from(idx))
    }

    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a V> {
        self.data.iter()
    }
}
