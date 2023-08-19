use core::marker::PhantomData;
use crate::util::sync::SpinMutex;

pub struct Singleton<T, I> {
    value: SpinMutex<Option<T>>,
    _phantom: PhantomData<I>
}