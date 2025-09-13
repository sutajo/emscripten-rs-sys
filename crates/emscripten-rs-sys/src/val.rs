use std::{
    ffi::CStr,
    mem::transmute,
    ptr::{self, null_mut},
};

use crate::*;

#[link(name = "embind")]
unsafe extern "C" {}

pub type RawEmval = *mut emscripten__EM_VAL;

#[repr(transparent)]
#[derive(PartialEq, Eq, Debug)]
struct Emval {
    inner: RawEmval,
}

enum EmType {
    Array,
    Object,
    String,
    Number,
}

impl Emval {
    pub fn from_raw(raw: RawEmval) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            emscripten_internal__emval_incref(raw);
        }
        Some(Emval { inner: raw })
    }

    pub fn type_of(&self) -> Option<Emval> {
        Emval::from_raw(unsafe { emscripten_internal__emval_typeof(self.inner) })
    }

    pub fn global(global: &CStr) -> Option<Emval> {
        Emval::from_raw(unsafe { emscripten_internal__emval_get_global(global.as_ptr()) })
    }

    pub fn module_property(prop: &CStr) -> Option<Emval> {
        Emval::from_raw(unsafe { emscripten_internal__emval_get_module_property(prop.as_ptr()) })
    }

    pub fn is_number(&self) -> bool {
        unsafe { emscripten_internal__emval_is_number(self.inner) }
    }

    pub fn is_string(&self) -> bool {
        unsafe { emscripten_internal__emval_is_string(self.inner) }
    }
}

impl From<&CStr> for Emval {
    fn from(value: &CStr) -> Self {
        Emval::from_raw(unsafe { emscripten_internal__emval_new_cstring(value.as_ptr()) }).unwrap()
    }
}

impl From<&Emval> for i32 {
    fn from(value: &Emval) -> Self {
        unsafe {
            let args = [118632, 118632];
            let invoker = emscripten_internal__emval_create_invoker(
                args.len() as _,
                args.as_ptr() as _,
                emscripten_internal_EM_INVOKER_KIND_CAST,
            );
            let argv = [118632, 0];
            emscripten_internal__emval_invoke(
                invoker,
                value.inner,
                ptr::null(),
                ptr::null_mut(),
                argv.as_ptr() as _,
            )
            .to_bits()
            .cast_signed() as _
        }
    }
}

impl Drop for Emval {
    fn drop(&mut self) {
        unsafe {
            emscripten_internal__emval_decref(self.inner);
        }
    }
}

impl Clone for Emval {
    fn clone(&self) -> Self {
        unsafe {
            emscripten_internal__emval_incref(self.inner);
        }
        Emval { inner: self.inner }
    }
}

struct EmIterator {
    current: Emval,
}

impl IntoIterator for Emval {
    type Item = Emval;
    type IntoIter = EmIterator;

    fn into_iter(self) -> Self::IntoIter {
        EmIterator { current: self }
    }
}

impl Iterator for EmIterator {
    type Item = Emval;

    fn next(&mut self) -> Option<Self::Item> {
        let next = unsafe { emscripten_internal__emval_iter_next(self.current.inner) };
        Emval::from_raw(next)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        em_js::js_internal, emscripten_internal__emval_new_object,
        emscripten_internal__emval_set_property,
        val::{Emval, RawEmval},
    };

    js_internal!{
        fn return_number() -> RawEmval,
        {
            return Emval.toHandle(354)
        }
    }

    #[test]
    fn test_return_number() {
        let number = Emval::from_raw(unsafe { return_number() }).unwrap();
        assert!(number.is_number());
        assert!(number.type_of().unwrap().is_string())
    }

    js_internal!{
        fn return_iterator() -> RawEmval,
        return Emval.toHandle([1,2,3,4].entries())
    }

    #[test]
    fn test_return_iterator() {
        let val = Emval::from_raw(unsafe { return_iterator() }).unwrap();
        assert_eq!(val.into_iter().count(), 4);

        unsafe {
            let obj = emscripten_internal__emval_new_object();
            emscripten_internal__emval_set_property(obj, obj, obj);
        }
    }
}
