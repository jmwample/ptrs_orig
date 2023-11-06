use crate::Result;

use std::io::{Read, Write};

pub trait StreamHandler = for<'a, 'b> FnMut(&'a mut dyn Read, &'b mut dyn Write) -> Result<u64>;

pub fn from_transform<F>(mut transform: F) -> Result<Box<dyn StreamHandler>>
where
    F: FnMut(&mut dyn Read, &mut [u8]) -> Result<usize> + 'static,
{
    Ok(Box::new(
        move |r: &mut dyn Read, w: &mut dyn Write| -> Result<u64> {
            let mut buf = [0_u8; 1024];
            let mut out = [0_u8; 1024];
            let mut total = 0_u64;
            loop {
                let nr = r.read(&mut buf)?;
                if nr == 0 {
                    break;
                }
                let nw = transform(&mut &buf[..nr], &mut out)?;
                w.write_all(&out[..nw])?;
                total += nw as u64;
            }
            Ok(total)
        },
    ))
}

///
///						 write 	 =================>    encode   =================>   decode
///        [ loop Buffer ] -> | source | -> | encoding | -> | encoded | -> | decoding | -> | /dev/null |
///									    pipe						   pipe
///
#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::pipe_set; // {pipe_set, pipes}

    use std::thread;

    // #[test] // Disabled
    #[allow(non_snake_case, unused)]
    fn end_to_end_1_MB() -> Result<()> {
        let ((mut source, mut encoding), (mut encoded, mut decoding)) = pipe_set()?;

        let out_file = std::fs::File::create("/dev/null")?;
        let mut out_file = std::io::BufWriter::new(out_file);

        thread::spawn(move || {
            let mut stream_encode = from_transform(|r, mut w| {
                let mut buf = [0_u8; 1024];
                let nr = r.read(&mut buf)?;
                let nw = w.write(&buf[..nr])?;
                thread::sleep(std::time::Duration::from_millis(10));
                Ok(nw)
            })
            .expect("failed to build steam_encoder");

            stream_encode(&mut encoding, &mut encoded)
                .expect("failed while encoding the stream: {}");
        });

        thread::spawn(move || {
            let mut stream_decode = from_transform(|r, mut w| {
                let mut buf = [0_u8; 1024];
                let nr = r.read(&mut buf)?;
                let nw = w.write(&buf[..nr])?;
                Ok(nw)
            })
            .expect("failed to build steam_decoder");

            stream_decode(&mut decoding, &mut out_file)
                .expect("failed while decoding the stream: {}");
        });

        let write_me = vec![0_u8; 1024];
        let mut total: usize = 0;
        for _ in 0..1024 {
            total += source.write(&write_me)?;
        }

        assert_eq!(total, 1024 * 1024);

        Ok(())
    }
}
