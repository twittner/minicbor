//! Deprecated encodings of various types, kept for compatibility reasons.

use core::ops::Deref;
use crate::{Encoder, Decoder, Encode, Decode};
use crate::decode::Error as DecodeError;
use crate::encode::{Error as EncodeError, Write};

/// Newtype of [`std::net::IpAddr`] with a suboptimal encoding.
///
/// This is the encoding used in minicbor versions < 0.15.0
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IpAddr(pub std::net::IpAddr);

/// Newtype of [`std::net::Ipv4Addr`] with a suboptimal encoding.
///
/// This is the encoding used in minicbor versions < 0.15.0
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ipv4Addr(pub std::net::Ipv4Addr);

/// Newtype of [`std::net::Ipv6Addr`] with a suboptimal encoding.
///
/// This is the encoding used in minicbor versions < 0.15.0
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ipv6Addr(pub std::net::Ipv6Addr);

/// Newtype of [`std::net::SocketAddr`] with a suboptimal encoding.
///
/// This is the encoding used in minicbor versions < 0.15.0
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SocketAddr(pub std::net::SocketAddr);

/// Newtype of [`std::net::SocketAddrV4`] with a suboptimal encoding.
///
/// This is the encoding used in minicbor versions < 0.15.0
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SocketAddrV4(pub std::net::SocketAddrV4);

/// Newtype of [`std::net::SocketAddrV6`] with a suboptimal encoding.
///
/// This is the encoding used in minicbor versions < 0.15.0
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SocketAddrV6(pub std::net::SocketAddrV6);

impl Encode for IpAddr {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), EncodeError<W::Error>> {
        e.array(2)?;
        match self.0 {
            std::net::IpAddr::V4(a) => { e.u32(0)?; Ipv4Addr(a).encode(e) }
            std::net::IpAddr::V6(a) => { e.u32(1)?; Ipv6Addr(a).encode(e) }
        }
    }
}

impl Encode for Ipv4Addr {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), EncodeError<W::Error>> {
        self.0.octets().encode(e)
    }
}

impl Encode for Ipv6Addr {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), EncodeError<W::Error>> {
        self.0.octets().encode(e)
    }
}

impl Encode for SocketAddr {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), EncodeError<W::Error>> {
        e.array(2)?;
        match self.0 {
            std::net::SocketAddr::V4(a) => { e.u32(0)?; SocketAddrV4(a).encode(e) }
            std::net::SocketAddr::V6(a) => { e.u32(1)?; SocketAddrV6(a).encode(e) }
        }
    }
}

impl Encode for SocketAddrV4 {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), EncodeError<W::Error>> {
        e.array(2)?
            .encode(Ipv4Addr(*self.0.ip()))?
            .encode(self.0.port())?
            .ok()
    }
}

impl Encode for SocketAddrV6 {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), EncodeError<W::Error>> {
        e.array(2)?
            .encode(Ipv6Addr(*self.0.ip()))?
            .encode(self.0.port())?
            .ok()
    }
}

macro_rules! decode_fields {
    ($d:ident | $($n:literal $x:ident => $t:ty ; $msg:literal)*) => {
        $(let mut $x : core::option::Option<$t> = None;)*

        let p = $d.position();

        match $d.array()? {
            Some(n) => for i in 0 .. n {
                match i {
                    $($n => $x = Some(Decode::decode($d)?),)*
                    _    => $d.limited_skip()?
                }
            }
            None => {
                let mut i = 0;
                while $d.datatype()? != crate::data::Type::Break {
                    match i {
                        $($n => $x = Some(Decode::decode($d)?),)*
                        _    => $d.limited_skip()?
                    }
                    i += 1
                }
                $d.limited_skip()?
            }
        }

        $(let $x = if let Some(x) = $x {
            x
        } else {
            return Err(DecodeError::missing_value($n).at(p).with_message($msg))
        };)*
    }
}

impl<'b> Decode<'b> for IpAddr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, DecodeError> {
        let p = d.position();
        if Some(2) != d.array()? {
            return Err(DecodeError::message("expected enum (2-element array)").at(p))
        }
        let p = d.position();
        match d.u32()? {
            0 => Ok(IpAddr(Ipv4Addr::decode(d)?.0.into())),
            1 => Ok(IpAddr(Ipv6Addr::decode(d)?.0.into())),
            n => Err(DecodeError::unknown_variant(n).at(p))
        }
    }
}

impl<'b> Decode<'b> for Ipv4Addr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, DecodeError> {
        let octets: [u8; 4] = Decode::decode(d)?;
        Ok(Ipv4Addr(octets.into()))
    }
}

impl<'b> Decode<'b> for Ipv6Addr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, DecodeError> {
        let octets: [u8; 16] = Decode::decode(d)?;
        Ok(Ipv6Addr(octets.into()))
    }
}

impl<'b> Decode<'b> for SocketAddr {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, DecodeError> {
        let p = d.position();
        if Some(2) != d.array()? {
            return Err(DecodeError::message("expected enum (2-element array)").at(p))
        }
        let p = d.position();
        match d.u32()? {
            0 => Ok(SocketAddr(SocketAddrV4::decode(d)?.0.into())),
            1 => Ok(SocketAddr(SocketAddrV6::decode(d)?.0.into())),
            n => Err(DecodeError::unknown_variant(n).at(p))
        }
    }
}

impl<'b> Decode<'b> for SocketAddrV4 {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, DecodeError> {
        decode_fields! { d |
            0 ip   => Ipv4Addr ; "SocketAddrV4::ip"
            1 port => u16      ; "SocketAddrV4::port"
        }
        Ok(SocketAddrV4(std::net::SocketAddrV4::new(ip.0, port)))
    }
}

impl<'b> Decode<'b> for SocketAddrV6 {
    fn decode(d: &mut Decoder<'b>) -> Result<Self, DecodeError> {
        decode_fields! { d |
            0 ip   => Ipv6Addr ; "SocketAddrV6::ip"
            1 port => u16      ; "SocketAddrV6::port"
        }
        Ok(SocketAddrV6(std::net::SocketAddrV6::new(ip.0, port, 0, 0)))
    }
}

impl Deref for IpAddr {
    type Target = std::net::IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Ipv4Addr {
    type Target = std::net::Ipv4Addr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Ipv6Addr {
    type Target = std::net::Ipv6Addr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SocketAddr {
    type Target = std::net::SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SocketAddrV4 {
    type Target = std::net::SocketAddrV4;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SocketAddrV6 {
    type Target = std::net::SocketAddrV6;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

