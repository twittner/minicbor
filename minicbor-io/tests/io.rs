use minicbor::bytes::ByteSlice;
use minicbor::encode::write::{self, Cursor, Write};
use minicbor_io::{Reader, Writer};
use std::io;

quickcheck::quickcheck! {
    fn read_write_bytes_identity(data: Vec<u8>) -> bool {
        let mut d = data;
        d.truncate(512 * 1024);

        let mut w = Writer::new(Vec::new());
        let val: &ByteSlice = d.as_slice().into();
        w.write(val).unwrap();

        let mut r = Reader::new(io::Cursor::new(w.into_parts().0));
        let val: &ByteSlice = r.read().unwrap().unwrap();
        val.as_ref() == d.as_slice()
    }

    fn read_write_num_identity(num: u64) -> bool {
        let mut w = Writer::new(Vec::new());
        w.write(num).unwrap();

        let mut r = Reader::new(io::Cursor::new(w.into_parts().0));
        let val: u64 = r.read().unwrap().unwrap();
        val == num
    }

    fn cursor_write(buf: Vec<u8>, data: Vec<u8>) -> bool {
        cursor_write_impl(buf, data)
    }
}

// quickcheck macro does not accept mutable parameters
fn cursor_write_impl(mut buf: Vec<u8>, data: Vec<u8>) -> bool {
    let mut c = Cursor::new(&mut buf[..]);
    if data.len() > c.get_ref().len() {
        if let Err(e) = c.write_all(&data) {
            let _: write::EndOfSlice = e;
            return true
        } else {
            return false
        }
    } else {
        assert!(c.write_all(&data).is_ok());
        data == c.get_ref()[.. c.position()]
    }
}
