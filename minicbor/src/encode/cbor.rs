#[macro_export]
macro_rules! cbor {
    // Logarithmic counting macro
    // https://users.rust-lang.org/t/logarithmic-counting-macro/8918
    (@count)         => { 0usize };
    (@count $one:tt) => { 1usize };
    (@count $($pairs:tt $_:tt)*)  => { cbor!(@count $($pairs)*) << 1usize };
    (@count $odd:tt $($rest:tt)*) => { cbor!(@count $($rest)*) | 1usize };

    ($x:tt) => {
        move |e: &mut $crate::encode::Encoder<_>| { cbor!(e, $x) }
    };

    ($e:expr, {$($i:tt $_:tt : $v:tt),*}) => {{
        let n = cbor!(@count $($i)*);
        $e.map(n)?;
        $(cbor!($e, $i)?; cbor!($e, $v)?;)*;
        $e.ok()
    }};
    ($e:expr, {$($k:tt : $v:tt),*}) => {{
        let n = cbor!(@count $($k)*);
        $e.map(n)?;
        $(cbor!($e, $k)?; cbor!($e, $v)?;)*;
        $e.ok()
    }};
    ($e:expr, [$($x:tt),*]) => {{
        let n = cbor!(@count $($x)*);
        $e.array(n)?;
        $(cbor!($e, $x)?;)*;
        $e.ok()
    }};
    ($e:expr, null)      => {{ $e.null()?.ok() }};
    ($e:expr, undefined) => {{ $e.undefined()?.ok() }};
    ($e:expr, $x:tt)     => {{
        $crate::encode::Encode::encode(&$x, $e)?;
        $e.ok()
    }}
}
