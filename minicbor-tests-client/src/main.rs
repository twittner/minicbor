#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "derive")]
mod derive {
    use minicbor::{Encode, Decode};

    #[derive(Encode, Decode)]
    struct S<'a> {
        #[b(0)] a: &'a str,
        #[cfg(feature = "alloc")]
        #[b(1)] b: alloc::borrow::Cow<'a, str>,
        #[cfg(feature = "std")]
        #[b(2)] c: std::borrow::Cow<'a, str>
    }
}

fn main() {
}
