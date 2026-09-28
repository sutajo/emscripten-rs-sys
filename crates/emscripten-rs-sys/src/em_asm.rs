use std::ffi::c_char;

pub use crate::{emscripten_asm_const_double, emscripten_asm_const_int, emscripten_asm_const_ptr};
pub use emscripten_rs_macros::js_asm;

pub trait AsmSignature: Default {
    const SIGNATURE: char;
}

macro_rules! int_like {
    ($($t:ty),*) => {
        $(
            impl AsmSignature for $t {
                const SIGNATURE: char = 'i';
            }
        )*
    };
}

int_like!(u8, i8, u16, i16, u32, i32, ());

impl AsmSignature for i64 {
    const SIGNATURE: char = 'j';
}

impl AsmSignature for u64 {
    const SIGNATURE: char = 'j';
}

impl<T> AsmSignature for *const T {
    const SIGNATURE: char = 'p';
}

impl<T> AsmSignature for *mut T {
    const SIGNATURE: char = 'p';
}

impl AsmSignature for f32 {
    const SIGNATURE: char = 'f';
}

impl AsmSignature for f64 {
    const SIGNATURE: char = 'd';
}

/// Builds a null-terminated EM_ASM argument signature on stable Rust.
/// `N` is the buffer capacity, including the terminating null byte.
pub struct SignatureBuilder<const N: usize> {
    sig: [c_char; N],
    len: usize,
}

impl<const N: usize> SignatureBuilder<N> {
    /// The return type is accepted for compatibility but is not encoded:
    /// Emscripten's signature describes only the variadic arguments.
    pub const fn new<Ret: AsmSignature>() -> Self {
        assert!(N > 0, "signature needs space for a null terminator");
        Self {
            sig: [0; N],
            len: 0,
        }
    }

    pub const fn new_for<Ret: AsmSignature>(_: &Ret) -> Self {
        Self::new::<Ret>()
    }

    pub const fn add_param<Param: AsmSignature>(mut self, _: &Param) -> Self {
        assert!(self.len + 1 < N, "signature buffer is full");
        self.sig[self.len] = Param::SIGNATURE as c_char;
        self.len += 1;
        self
    }

    pub const fn finish(self) -> [c_char; N] {
        self.sig
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_ints() {
        let x = 10;
        let y = 20;
        let result = js_asm! { |x,y| -> i32 { return x + y; } };
        assert_eq!(result, 30);
    }

    #[test]
    fn basic() {
        let number = 2;
        js_asm! {
            |number| {
                console.log("Got: ", number);
            }
        }

        assert_eq!(
            js_asm! {
                || -> i32 {
                    return 1;
                }
            },
            1
        );
    }

    #[test]
    fn mul_int() {
        let x = 3;
        let y = 2;

        let result = js_asm! {
            |x,y| -> i32 {
                return x*y;
            }
        };
        assert_eq!(result, 6);
    }

    #[test]
    fn add_doubles() {
        let a = 12.5;
        let b = 52.2;

        let result = js_asm! {
            |a,b| -> f64 {
                return a*b;
            }
        };
        assert_eq!(result, 12.5 * 52.2);
    }

    #[test]
    fn ptr_return() {
        let string_ptr = c"Hello".as_ptr();
        let returned_ptr = js_asm! { |string_ptr| -> *mut c_char { return string_ptr; } };
        assert_eq!(string_ptr, returned_ptr as *const c_char);
    }
}
