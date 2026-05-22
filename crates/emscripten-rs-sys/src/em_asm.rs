use std::ffi::{c_char};

pub use crate::{emscripten_asm_const_double, emscripten_asm_const_ptr, emscripten_asm_const_int};
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

int_like!(
    u8, i8,
    u16, i16,
    u32, i32
);

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

impl AsmSignature for () {
    const SIGNATURE: char = 'i';
}

pub struct SignatureBuilder<const N: usize> {
    sig: [c_char; N],
}

impl SignatureBuilder<1> {
    pub const fn new<Ret: AsmSignature>() -> Self {
        Self {
            sig: [Ret::SIGNATURE as c_char],
        }
    }

     pub const fn new_for<Ret: AsmSignature>(_: &Ret) -> Self {
        Self {
            sig: [Ret::SIGNATURE as c_char],
        }
    }
}

const fn push<T: Copy + [const] Default, const N: usize>(arr: [T; N], value: T) -> [T; N + 1]
{
    let mut out = [T::default(); N + 1];
    let _ = &out[..N].copy_from_slice(&arr);
    out[N] = value;
    out
}

impl<const N: usize> SignatureBuilder<N> {
    pub const fn add_param<Param: AsmSignature>(self, _: &Param) -> SignatureBuilder<{ N + 1 }> {
        SignatureBuilder {
            sig: push(self.sig, Param::SIGNATURE as c_char),
        }
    }

    pub const fn finish(self) -> [c_char; N + 1] {
        push(self.sig, '\0' as _)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_of_concept() {
        use crate::binding;

        let result = unsafe {
            unsafe extern "C" {
                pub unsafe static CODE: [u8; 10];
            }

            let x = 10;
            let y = 20;

            mod generated {
                std::arch::global_asm!(
                    ".section em_asm,\"R\",@",
                    ".p2align 0",
                    ".globl CODE",
                    ".type CODE,@object",
                    "CODE:",
                    ".asciz \"return $0 + $1;\""
                );
            }
            binding::emscripten_asm_const_int(
                CODE.as_ptr() as _,
                SignatureBuilder::new::<i32>()
                    .add_param(&x)
                    .add_param(&y)
                    .finish()
                    .as_ptr(),
                x,
                y,
            )
        };

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
    fn ptr_return()
    {
        let string_ptr = c"Hello".as_ptr();
        let returned_ptr = js_asm!{ |string_ptr| -> *mut c_char { return string_ptr; } };
        assert_eq!(string_ptr, returned_ptr as *const c_char);
    }
}
