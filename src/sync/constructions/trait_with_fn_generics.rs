
use std::io::{self, Read, Result, Write};

use crate::constructions::Named;

// type transform_fn =
// type StreamHandle = dyn for<'a, 'b> FnMut(&'a mut (dyn Read + 'a), &'b mut (dyn Write + 'b)) -> Result<u64>;

pub trait Transport: Named {

    fn listen_handler<E, D>(&self, _remote: &mut (impl Read + Write)) -> Result<(E, D)>
    where
        E: for<'a, 'b> FnMut(&'a mut (dyn Read + 'a), &'b mut (dyn Write + 'b)) -> Result<u64>
            + 'static,
        D: for<'a, 'b> FnMut(&'a mut (dyn Read + 'a), &'b mut (dyn Write + 'b)) -> Result<u64>
            + 'static,
    {
        Ok((io::copy, io::copy))
    }

    fn dial_handler<E, D>(&self, _remote: &mut (impl Read + Write)) -> Result<(E, D)>
    where
        E: for<'a, 'b> FnMut(&'a mut (dyn Read + 'a), &'b mut (dyn Write + 'b)) -> Result<u64>
            + 'static,
        D: for<'a, 'b> FnMut(&'a mut (dyn Read + 'a), &'b mut (dyn Write + 'b)) -> Result<u64>
            + 'static,
    {
        Ok((io::copy, io::copy))
    }
}
