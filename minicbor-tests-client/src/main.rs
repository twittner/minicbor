#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "derive")]
mod derive {
    use minicbor::{Encode, Decode};

    #[derive(Encode, Decode)]
    #[allow(unused)]
    struct S<'a> {
        #[b(0)] a: &'a str,
        #[cfg(feature = "alloc")]
        #[b(1)] b: alloc::borrow::Cow<'a, str>,
        #[cfg(feature = "std")]
        #[b(2)] c: std::borrow::Cow<'a, str>
    }

    #[derive(Encode, Decode)]
    #[cbor(max_depth(8))]
    struct D<'a> {
        #[b(0)] a: &'a str,
        #[n(1)] b: Option<u32>
    }

    #[derive(Encode, Decode)]
    #[cbor(max_depth(8))]
    enum E {
        #[n(0)] A(#[n(0)] u8)
    }
}

fn main() {
}
