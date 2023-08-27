
pub struct UnresizableArray<T, const TCapacity: usize>
where
    T: Sized,
{
    data: Box<[T; TCapacity]>,
    current_index: usize,
}

impl<T, const TCapacity: usize> UnresizableArray<T, TCapacity>
where
    T: Sized,
{
    const MAX_SIZE: usize = TCapacity;
    pub fn with_capacity() -> Self
    where
        [(); Self::MAX_SIZE]:,
    {
        unsafe {
            Self {
                data: Box::new_uninit().assume_init(),
                current_index: 0,
            }
        }
    }

    pub fn push(&mut self, mut item: T) -> *const T {
        if self.current_index >= Self::MAX_SIZE {
            panic!("UnresizableArray is out of memory");
        }

        std::mem::swap(&mut self.data[self.current_index], &mut item);
        std::mem::forget(item);

        let ptr = &self.data[self.current_index] as *const T;
        self.current_index += 1;
        return ptr;
    }

    pub fn get_unchecked(&self, index: usize) -> *const T {
        assert!(index < self.current_index);
        &self.data[index]
    }
}
