use byteorder::{ReadBytesExt, LE};

pub trait ReadExt {
    fn read_bool(&mut self) -> Result<bool, super::Error>;
    fn read_guid(&mut self) -> Result<[u8; 20], super::Error>;
    fn read_array<T>(
        &mut self,
        func: impl FnMut(&mut Self) -> Result<T, super::Error>,
    ) -> Result<Vec<T>, super::Error>;
    fn read_string(&mut self) -> Result<String, super::Error>;
    fn read_name(&mut self) -> Result<String, super::Error>;
    fn read_len(&mut self, len: usize) -> Result<Vec<u8>, super::Error>;
}

impl<R: std::io::Read> ReadExt for R {
    fn read_bool(&mut self) -> Result<bool, super::Error> {
        match self.read_u8()? {
            1 => Ok(true),
            0 => Ok(false),
            err => Err(super::Error::Bool(err)),
        }
    }

    fn read_guid(&mut self) -> Result<[u8; 20], super::Error> {
        let mut guid = [0; 20];
        self.read_exact(&mut guid)?;
        Ok(guid)
    }

    fn read_array<T>(
        &mut self,
        mut func: impl FnMut(&mut Self) -> Result<T, super::Error>,
    ) -> Result<Vec<T>, super::Error> {
        let mut buf = Vec::with_capacity(self.read_u32::<LE>()? as usize);
        for _ in 0..buf.capacity() {
            buf.push(func(self)?);
        }
        Ok(buf)
    }

    fn read_string(&mut self) -> Result<String, crate::Error> {
        let mut buf = match self.read_i32::<LE>()? {
            size if size.is_negative() => {
                let mut buf = Vec::with_capacity(-size as usize);
                for _ in 0..buf.capacity() {
                    buf.push(self.read_u16::<LE>()?);
                }
                String::from_utf16(&buf)?
            }
            size => String::from_utf8(self.read_len(size as usize)?)?,
        };
        // remove the null byte
        buf.pop();
        Ok(buf)
    }

    fn read_name(&mut self) -> Result<String, crate::Error> {
        let mut path = self.read_string()?;
        if !path.starts_with("Engine") {
            if let Some(pos) = path.find("Content") {
                path.replace_range(0..pos + 7, "Game");
            }
        }
        Ok(format!("/{}", path.trim_start_matches('/')))
    }

    fn read_len(&mut self, len: usize) -> Result<Vec<u8>, super::Error> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}
