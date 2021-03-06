use super::Error;
use crate::{
    borrowed::{parse, parse::NL, Signature},
    commit, owned, BStr, ByteSlice,
};
use nom::{
    branch::alt,
    bytes::{complete::is_not, complete::tag},
    combinator::{all_consuming, opt},
    multi::many0,
    IResult,
};
use smallvec::SmallVec;
use std::borrow::Cow;

#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Commit<'a> {
    // HEX SHA1 of tree object we point to
    #[cfg_attr(feature = "serde1", serde(borrow))]
    pub tree: &'a BStr,
    /// HEX SHA1 of each parent commit. Empty for first commit in repository.
    pub parents: SmallVec<[&'a BStr; 1]>,
    pub author: Signature<'a>,
    pub committer: Signature<'a>,
    /// The name of the message encoding, otherwise UTF-8 should be assumed.
    pub encoding: Option<&'a BStr>,
    pub message: &'a BStr,
    /// Extra header fields, either single line or multi-line.
    pub extra_headers: Vec<(&'a BStr, Cow<'a, BStr>)>,
}

fn parse_message(i: &[u8]) -> IResult<&[u8], &BStr, Error> {
    if i.is_empty() {
        // newline + [message]
        return Err(nom::Err::Error(Error::NomDetail(i.into(), "commit message is missing")));
    }
    let (i, _) = tag(NL)(i).map_err(Error::context("a newline separates headers from the message"))?;
    debug_assert!(!i.is_empty());
    Ok((&[], &i.as_bstr()))
}

fn parse(i: &[u8]) -> IResult<&[u8], Commit, Error> {
    let (i, tree) =
        parse::header_field(i, b"tree", parse::hex_sha1).map_err(Error::context("tree <40 lowercase hex char>"))?;
    let (i, parents) = many0(|i| parse::header_field(i, b"parent", parse::hex_sha1))(i)
        .map_err(Error::context("zero or more 'parent <40 lowercase hex char>'"))?;
    let (i, author) =
        parse::header_field(i, b"author", parse::signature).map_err(Error::context("author <signature>"))?;
    let (i, committer) =
        parse::header_field(i, b"committer", parse::signature).map_err(Error::context("committer <signature>"))?;
    let (i, encoding) =
        opt(|i| parse::header_field(i, b"encoding", is_not(NL)))(i).map_err(Error::context("encoding <encoding>"))?;
    let (i, extra_headers) = many0(alt((
        |i| parse::any_header_field_multi_line(i).map(|(i, (k, o))| (i, (k.as_bstr(), Cow::Owned(o)))),
        |i| parse::any_header_field(i, is_not(NL)).map(|(i, (k, o))| (i, (k.as_bstr(), Cow::Borrowed(o.as_bstr())))),
    )))(i)
    .map_err(Error::context("<field> <single-line|multi-line>"))?;
    let (i, message) = all_consuming(parse_message)(i)?;

    Ok((
        i,
        Commit {
            tree,
            parents: SmallVec::from(parents),
            author,
            committer,
            encoding: encoding.map(ByteSlice::as_bstr),
            message,
            extra_headers,
        },
    ))
}

impl<'a> Commit<'a> {
    pub fn tree(&self) -> owned::Id {
        owned::Id::from_40_bytes_in_hex(self.tree).expect("prior validation")
    }
    pub fn from_bytes(d: &'a [u8]) -> Result<Commit<'a>, Error> {
        parse(d).map(|(_, t)| t).map_err(Error::from)
    }
    pub fn extra_headers(&self) -> commit::ExtraHeaders<impl Iterator<Item = (&BStr, &BStr)>> {
        commit::ExtraHeaders::new(self.extra_headers.iter().map(|(k, v)| (*k, v.as_ref())))
    }
}
