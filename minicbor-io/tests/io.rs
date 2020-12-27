use minicbor::bytes::ByteSlice;
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
}
