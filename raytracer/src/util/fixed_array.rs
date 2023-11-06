
pub struct FixedArray<T, const TCAPACITY: usize>
where
    T: Sized
{
    data: Box<[T; TCAPACITY]>,
    current_index: usize,
}

impl<T, const TCAPACITY: usize> FixedArray<T, TCAPACITY>
where
    T: Sized,
{
    const MAX_ELEMENTS: usize = TCAPACITY;
    pub fn with_capacity() -> Self
    where
        [(); TCAPACITY]:,
    {
        unsafe {
            Self {
                data: Box::new_uninit().assume_init(),
                current_index: 0,
            }
        }
    }

    pub fn push(&mut self, mut item: T) -> *mut T {
        if self.current_index >= Self::MAX_ELEMENTS {
            panic!("FixedArray is out of memory");
        }

        std::mem::swap(&mut self.data[self.current_index], &mut item);
        std::mem::forget(item);

        let ptr = &mut self.data[self.current_index] as *mut T;
        self.current_index += 1;
        return ptr;
    }

    pub fn get_unchecked(&self, index: usize) -> *const T {
        debug_assert!(index < self.current_index);
        &self.data[index]
    }

    pub fn memory_info(&self) -> (usize, usize) {
        return (self.current_index, Self::MAX_ELEMENTS);
    }
}
