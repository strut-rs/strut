use std::convert::Infallible;
use strut_rabbitmq::Decoder;

pub struct TrivialDecoder;

impl Decoder for TrivialDecoder {
    type Result = String;
    type Error = Infallible;

    fn decode(&self, bytes: &[u8]) -> Result<Self::Result, Self::Error> {
        Ok(String::from_utf8_lossy(bytes).to_string())
    }
}
