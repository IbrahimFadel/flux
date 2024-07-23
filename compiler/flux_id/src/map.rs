use std::marker::PhantomData;

#[derive(Debug, Clone)]
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

impl<K, V> Map<K, V>
where
    K: From<usize> + Into<usize>,
{
    pub fn new() -> Self {
        Self {
            data: vec![],
            _idx: PhantomData,
        }
    }

    pub fn insert(&mut self, v: V) -> K {
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

    pub fn get_mut(&mut self, k: K) -> &mut V
    where
        K: Into<usize>,
    {
        &mut self.data[k.into()]
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

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (K, &'a V)> {
        self.data
            .iter()
            .enumerate()
            .map(|(idx, v)| (K::from(idx), v))
    }
}
