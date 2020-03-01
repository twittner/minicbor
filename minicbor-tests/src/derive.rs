use minicbor::{Decode, Encode};

#[derive(Decode, Encode)]
pub struct User<'a, T> {
    #[n(1)] name: &'a str,
    #[n(2)] age: T,
    #[n(3)] addr: Address
}

#[derive(Decode, Encode)]
pub struct Address {
    #[n(1)] street: String,
    #[n(2)] postcode: u32,
    #[n(3)] points: Vec<Point>,
    #[n(4)] extras: Option<bool>,
    #[n(5)] maybe: Maybe,
    #[n(6)] oneof: OneOf
}

#[derive(Decode, Encode)]
pub struct Point(#[n(1)] u64, #[n(2)] u64);

#[derive(Decode, Encode)]
pub enum Maybe {
    #[n(1)] Yes,
    #[n(2)] No
}

#[derive(Decode, Encode)]
pub enum OneOf {
    #[n(1)] First(#[n(1)] bool, #[n(2)] Option<String>),
    #[n(2)] Second {
        #[n(1)] mother: Option<String>,
        #[n(2)] father: String
    }
}

#[derive(Decode, Encode)]
pub enum Bar<T, U> {
    #[n(1)] A(#[n(1)] T),
    #[n(2)] B(#[n(1)] U)
}

