use core::{
    hash::Hash,
    ops::Deref,
};

use crate::*;

use sync::OnceLock;

pub(crate) struct StaticMap<K: 'static, V: 'static> {
    pairs: &'static [(K, V)],
    map: OnceLock<HashMap<K, V>>,
}

impl<K, V> Deref for StaticMap<K, V>
    where
        K: Hash + Eq + Copy + 'static,
        V: Hash + Eq + Copy + 'static,
{

    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        self.map.get_or_init(||
            self.pairs
                .iter()
                .copied()
                .collect()
        )
    }
}

#[inline]
pub(crate) const fn static_map<K, V>(
    pairs: &'static [(K, V)],
) -> StaticMap<K, V>
    where
        K: Hash + Eq + Copy + 'static,
        V: Copy + 'static,
{
    StaticMap { pairs, map: OnceLock::new(), }
}
