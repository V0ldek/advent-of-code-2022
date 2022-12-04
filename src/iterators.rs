pub trait SplitIteratorExt: Iterator + Sized
where
    Self::Item: Eq,
{
    fn split(self, element: Self::Item) -> SplitIterator<Self>;
}

impl<I: Iterator> SplitIteratorExt for I
where
    I::Item: Eq,
{
    fn split(self, element: Self::Item) -> SplitIterator<Self> {
        SplitIterator {
            iter: self,
            element,
        }
    }
}

pub struct SplitIterator<I: Iterator>
where
    I::Item: Eq,
{
    iter: I,
    element: I::Item,
}

impl<I: Iterator> Iterator for SplitIterator<I>
where
    I::Item: Eq,
{
    type Item = SplitChunkIterator<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut vec = vec![];
        let mut element = self.iter.next();

        element.as_ref()?;

        loop {
            match element {
                None => break,
                Some(e) if e == self.element => break,
                Some(e) => {
                    vec.push(e);
                    element = self.iter.next();
                }
            }
        }

        Some(SplitChunkIterator {
            items: vec.into_iter(),
        })
    }
}

pub struct SplitChunkIterator<T> {
    items: std::vec::IntoIter<T>,
}

impl<T> Iterator for SplitChunkIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}
