mod reader;

use futures_core::Stream;
use futures_io::AsyncRead;
use futures_util::stream;

pub use reader::{Reader, Error as ReadError};

/// Decode a [`Stream`] of values from an [`AsyncRead`].
///
/// Stream elements are length-prefixed CBOR items.
pub fn stream<T, R>(r: R) -> impl Stream<Item = Result<T, ReadError>>
where
    T: for<'a> minicbor::Decode<'a>,
    R: AsyncRead + Unpin
{
    stream::unfold(Reader::new(r), |mut r| async move {
        let n = match r.read_len().await {
            Ok(Some(n)) => n,
            Ok(None)    => return None,
            Err(e)      => return Some((Err(e), r))
        };
        match r.read_val(n).await {
            Ok(v) => Some((Ok(v), r)),
            Err(e) => Some((Err(e), r))
        }
    })
}

