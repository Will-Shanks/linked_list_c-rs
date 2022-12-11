use core::ptr;
use libc::c_void;
use core::marker::PhantomData;

pub trait LlItem {
    fn get_next(&self) -> *mut Self;
    fn set_next(&mut self, next: *mut Self);
}

#[macro_export]
macro_rules! impl_LlItem {
    ([$($t:ty),+]) => {
        $(impl linked_list_c::LlItem for $t {
            fn get_next(&self) -> *mut Self {
                self.next
            }
            fn set_next(&mut self, next: *mut $t) {
                let old = self.next;
                if !old.is_null() && !next.is_null() { 
                    unsafe{(*next).set_next(old)};
                }
                self.next = next;
            }
        })*
    }
}

pub struct List<'a, T: LlItem> {
    head: *mut T,
    n: *mut T,
    drop_first: Option<fn(*mut T)>,
    _phantom: PhantomData<&'a mut T>
}

impl<T: LlItem> List<'_, T> {
    pub fn new() -> List<'static, T> {
        List{head: ptr::null_mut(), n: ptr::null_mut(), drop_first: Some(|x: *mut T| unsafe{std::ptr::drop_in_place(x)}), _phantom: PhantomData}
    }

    pub fn from_c(elem: *mut T) -> List<'static, T> {
        List{head: elem, n: ptr::null_mut(), drop_first: Some(|x: *mut T| unsafe{libc::free(x as *mut c_void)}), _phantom: PhantomData}
    }

    pub unsafe fn with_custom_drop(first: *mut T, drop_first: Option<fn(*mut T)>) -> List<'static, T> {
        List{head: first, n: ptr::null_mut(), drop_first: drop_first, _phantom: PhantomData}
    }

    pub fn add(&mut self, mut elem: Box<T>) {
        if !self.head.is_null() {
            elem.set_next(self.head);
        }
        //into_raw is crucial so elem isn't dropped
        self.head = Box::into_raw(elem);
    }
    pub unsafe fn head(&self) -> *mut T {
        self.head
    }
}

impl<'a, T: LlItem> Drop for List<'a, T>{
    fn drop(&mut self) {
        if let Some(d) = self.drop_first {
            d(self.head);
        }
    }
}

impl<'a, T: LlItem> Iterator for List<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head.is_null() {
            None
        } else if self.n.is_null() {
            self.n = self.head;
            Some(unsafe{&*self.n})
        } else if unsafe{& *self.n}.get_next().is_null() {
            self.n = ptr::null_mut();
            None
        } else { self.n = unsafe{& *self.n}.get_next();
        Some(unsafe{&*self.n})
        }
    }
}
