use minicbor::{Encode, Decode};
use minicbor_io::{AsyncReader, AsyncWriter, Error};
use quickcheck::{Arbitrary, Gen};
use rand::Rng;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
struct Record {
    #[n(0)] firstname: String,
    #[n(1)] lastname: String,
    #[n(2)] birthday: u32,
    #[n(3)] addresses: Vec<Address>
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
struct RecordView<'a> {
    #[b(0)] firstname: &'a str,
    #[b(1)] lastname: &'a str,
    #[n(2)] birthday: u32,
    #[b(3)] addresses: Vec<AddressView<'a>>
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
struct Address {
    #[n(0)] street: String,
    #[n(1)] houseno: String,
    #[n(2)] postcode: u32,
    #[n(3)] city: String,
    #[n(4)] country: String
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
struct AddressView<'a> {
    #[b(0)] street: &'a str,
    #[b(1)] houseno: &'a str,
    #[n(2)] postcode: u32,
    #[b(3)] city: &'a str,
    #[b(4)] country: &'a str
}

impl Arbitrary for Record {
    fn arbitrary(g: &mut Gen) -> Self {
        Record {
            firstname: Arbitrary::arbitrary(g),
            lastname: Arbitrary::arbitrary(g),
            birthday: Arbitrary::arbitrary(g),
            addresses: Arbitrary::arbitrary(g)
        }
    }
}

impl Arbitrary for Address {
    fn arbitrary(g: &mut Gen) -> Self {
        Address {
            street: Arbitrary::arbitrary(g),
            houseno: Arbitrary::arbitrary(g),
            postcode: Arbitrary::arbitrary(g),
            city: Arbitrary::arbitrary(g),
            country: Arbitrary::arbitrary(g)
        }
    }
}

/// Write `Record`s, read `RecordView`s and assert their structural equality.
#[tokio::test]
async fn read_write_identity() {
    let (addr, server) = server().await.unwrap();
    let join = tokio::spawn(echo::<Record>(server));

    let mut gen = Gen::new(20);
    let mut rng = rand::thread_rng();
    let rounds  = rng.gen_range(10 .. 30);

    for n in 0u8 .. rounds {
        let mut client = TcpStream::connect(addr).await.unwrap();
        let (reader, writer) = client.split();
        let mut reader = AsyncReader::new(reader.compat());
        let mut writer = AsyncWriter::new(writer.compat_write());

        for _ in 0u8 .. rng.gen_range(1 .. 50) {
            let a = Record::arbitrary(&mut gen);
            writer.write(Command::Value(&a)).await.unwrap();
            let b: RecordView<'_> = reader.read().await.unwrap().unwrap();
            assert_eq!(a.firstname, b.firstname);
            assert_eq!(a.lastname, b.lastname);
            assert_eq!(a.birthday, b.birthday);
            assert_eq!(a.addresses.len(), b.addresses.len());
            for (a, b) in a.addresses.iter().zip(b.addresses.iter()) {
                assert_eq!(a.street, b.street);
                assert_eq!(a.houseno, b.houseno);
                assert_eq!(a.postcode, b.postcode);
                assert_eq!(a.city, b.city);
                assert_eq!(a.country, b.country);
            }
        }

        if n == rounds - 1 {
            writer.write(Command::<Record>::Stop).await.unwrap();
        }
    }

    join.await.unwrap().unwrap()
}

#[derive(Debug, Encode, Decode)]
enum Command<T> {
    #[n(0)] Stop,
    #[n(1)] Value(#[n(0)] T)
}

/// Bind a server to a random port.
async fn server() -> io::Result<(SocketAddr, TcpListener)> {
    let l = TcpListener::bind("127.0.0.1:0").await?;
    let a = l.local_addr()?;
    Ok((a, l))
}

/// For each connection, read the `Command` and if a
/// `Command::Value`, send back the value.
async fn echo<T>(l: TcpListener) -> Result<(), Error>
where
    T: Encode<()> + for<'a> Decode<'a, ()>
{
    while let Ok((mut s, _)) = l.accept().await {
        let (r, w) = s.split();
        let mut r = AsyncReader::new(r.compat());
        let mut w = AsyncWriter::new(w.compat_write());
        loop {
            match r.read().await? {
                None => break,
                Some(Command::<T>::Stop) => return Ok(()),
                Some(Command::<T>::Value(v)) => { w.write(v).await?; }
            }
        }
    }
    Ok(())
}
