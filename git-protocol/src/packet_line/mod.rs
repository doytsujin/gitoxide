use bstr::BStr;
use std::io;

pub(crate) const U16_HEX_BYTES: usize = 4;
pub(crate) const MAX_DATA_LEN: usize = 65516;
pub(crate) const MAX_LINE_LEN: usize = MAX_DATA_LEN + U16_HEX_BYTES;
pub(crate) const FLUSH_LINE: &[u8] = b"0000";
pub(crate) const DELIMITER_LINE: &[u8] = b"0001";
pub(crate) const RESPONSE_END_LINE: &[u8] = b"0002";
pub(crate) const ERR_PREFIX: &[u8] = b"ERR ";

#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum Borrowed<'a> {
    Data(&'a [u8]),
    Flush,
    Delimiter,
    ResponseEnd,
}

impl<'a> Borrowed<'a> {
    pub fn to_write(&self, out: impl io::Write) -> Result<usize, encode::Error> {
        match self {
            Borrowed::Data(d) => encode::data_to_write(d, out),
            Borrowed::Flush => encode::flush_to_write(out).map_err(Into::into),
            Borrowed::Delimiter => encode::delim_to_write(out).map_err(Into::into),
            Borrowed::ResponseEnd => encode::response_end_to_write(out).map_err(Into::into),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Borrowed::Data(d) => d,
            Borrowed::Flush | Borrowed::Delimiter | Borrowed::ResponseEnd => &[],
        }
    }
    pub fn as_bstr(&self) -> &BStr {
        self.as_slice().into()
    }
    pub fn to_error(&self) -> Error {
        Error(self.as_slice())
    }
    pub fn to_band(&self, kind: Channel) -> Band {
        let d = match self {
            Borrowed::Data(d) => d,
            _ => panic!("cannot side-channel non-data lines"),
        };

        match kind {
            Channel::Data => Band::Data(d),
            Channel::Progress => Band::Progress(d),
            Channel::Error => Band::Error(d),
        }
    }
    /// Decode the band of the line, or panic if it is not actually a side-band line
    pub fn decode_band(&self) -> Band {
        let d = match self {
            Borrowed::Data(d) => d,
            _ => panic!("cannot decode side-channel information from non-data lines"),
        };
        match d[0] {
            1 => Band::Data(&d[1..]),
            2 => Band::Progress(&d[1..]),
            3 => Band::Error(&d[1..]),
            _ => panic!("attempt to decode a non-side channel line"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Error<'a>(&'a [u8]);

impl<'a> Error<'a> {
    pub fn to_write(&self, out: impl io::Write) -> Result<usize, encode::Error> {
        encode::error_to_write(self.0, out)
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum Band<'a> {
    Data(&'a [u8]),
    Progress(&'a [u8]),
    Error(&'a [u8]),
}

impl<'a> Band<'a> {
    pub fn to_write(&self, out: impl io::Write) -> Result<usize, encode::Error> {
        match self {
            Band::Data(d) => encode::band_to_write(Channel::Data, d, out),
            Band::Progress(d) => encode::band_to_write(Channel::Progress, d, out),
            Band::Error(d) => encode::band_to_write(Channel::Error, d, out),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum Channel {
    Data = 1,
    Progress = 2,
    Error = 3,
}

pub mod decode;
pub mod encode;
