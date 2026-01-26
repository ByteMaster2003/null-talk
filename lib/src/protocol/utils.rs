use std::io;

pub fn to_bytes<T>(value: &T) -> Vec<u8>
where
    T: serde::ser::Serialize,
{
    bincode::serialize(value).unwrap()
}

pub fn parse<'a, T>(bytes: &'a [u8]) -> Result<T, io::Error>
where
    T: serde::de::Deserialize<'a>,
{
    match bincode::deserialize::<T>(bytes) {
        Ok(msg) => return Ok(msg),
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
        }
    };
}
