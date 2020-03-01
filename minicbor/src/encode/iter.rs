use super::{Encode, Encoder, Error, Write};

#[derive(Debug, Clone)]
pub struct Iter<I>(I);

impl<I, T> From<I> for Iter<I>
where
    I: Iterator<Item = T> + Clone,
    T: Encode
{
    fn from(it: I) -> Self {
        Iter(it)
    }
}

impl<I, T> Encode for Iter<I>
where
    I: Iterator<Item = T> + Clone,
    T: Encode
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        let it = self.0.clone();
        e.begin_array()?;
        for x in it {
            x.encode(e)?
        }
        e.end()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ExactSizeIter<I>(I);

impl<I, T> From<I> for ExactSizeIter<I>
where
    I: ExactSizeIterator<Item = T> + Clone,
    T: Encode
{
    fn from(it: I) -> Self {
        ExactSizeIter(it)
    }
}

impl<I, T> Encode for ExactSizeIter<I>
where
    I: ExactSizeIterator<Item = T> + Clone,
    T: Encode
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), Error<W::Error>> {
        let it = self.0.clone();
        e.array(it.len())?;
        for x in it {
            x.encode(e)?
        }
        Ok(())
    }
}

