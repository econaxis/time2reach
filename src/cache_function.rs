use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

pub struct CacheDecorator<F, R>
where
    F: Fn(i64, i64) -> R,
{
    func: Mutex<F>,
    cache: Mutex<LruCache<u64, Arc<R>>>,
}

impl<F, R> CacheDecorator<F, R>
where
    F: Fn(i64, i64) -> R,
{
    pub fn new(func: F, capacity: usize) -> Self {
        CacheDecorator {
            func: Mutex::new(func),
            cache: Mutex::new(LruCache::new(NonZeroUsize::try_from(capacity).unwrap())),
        }
    }

    pub fn call(&self, arg1: i64, arg2: i64) -> Arc<R> {
        let mut hasher = DefaultHasher::new();
        (arg1, arg2).hash(&mut hasher);
        let key = hasher.finish();

        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(result) = cache.get(&key) {
                return result.clone();
            }
        }

        let result = (self.func.lock().unwrap())(arg1, arg2);

        let mut cache_lock = self.cache.lock().unwrap();
        cache_lock.put(key, result.into());

        cache_lock.get(&key).unwrap().clone()
    }
}

#[macro_use]
macro_rules! cache_decorator {
    ($func:expr, $capacity:expr) => {{
        let decorator = CacheDecorator::new($func, $capacity);
        move |arg1, arg2| decorator.call(arg1, arg2)
    }};
}
