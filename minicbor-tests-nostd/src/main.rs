#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{hprintln, debug};
use minicbor::{Encode, Decode, Encoder, Decoder};
use panic_halt as _;

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
struct SensorData {
    temperature: i16,
    humidity: u8,
    active: bool,
}

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
struct WithRename {
    #[cbor(key = "t")]
    temperature: i16,
}

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
struct WithOptional {
    name: u8,
    interval: Option<u32>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
enum Command {
    Reset,
    SetInterval { #[s("ms")] ms: u32 },
}

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
struct WithSkip {
    name: u8,
    #[cbor(skip)]
    cached: u32,
    value: u16,
}

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
struct WithArray {
    label: u8,
    readings: [u16; 3],
}

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
struct Empty {}

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
struct AllOpt {
    a: Option<u8>,
    b: Option<u16>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
#[cbor(text_keys)]
struct Generic<T> {
    value: T,
    label: u8,
}

fn assert(ok: bool, msg: &str) {
    if ok {
        hprintln!("  PASS: {}", msg);
    } else {
        hprintln!("  FAIL: {}", msg);
        debug::exit(debug::EXIT_FAILURE);
    }
}

fn encode_to_buf<T: Encode<()>>(val: &T, buf: &mut [u8]) -> usize {
    let buf_len = buf.len();
    let mut e = Encoder::new(&mut *buf);
    e.encode(val).unwrap();
    buf_len - e.writer().len()
}

#[entry]
fn main() -> ! {
    hprintln!("=== minicbor text_keys no_std tests on STM32H563ZI ===");

    let mut buf = [0u8; 128];

    // Test 1: Roundtrip SensorData
    {
        let val = SensorData { temperature: 23, humidity: 65, active: true };
        let len = encode_to_buf(&val, &mut buf);
        let decoded: SensorData = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "roundtrip SensorData");
    }

    // Test 2: Renamed keys
    {
        let val = WithRename { temperature: -5 };
        let len = encode_to_buf(&val, &mut buf);
        assert(buf[0] == 0xa1, "rename: map(1)");
        assert(buf[1] == 0x61, "rename: text(1)");
        assert(buf[2] == b't', "rename: key 't'");
        let decoded: WithRename = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "roundtrip WithRename");
    }

    // Test 3: Optional field absent
    {
        let val = WithOptional { name: 42, interval: None };
        let len = encode_to_buf(&val, &mut buf);
        assert(buf[0] == 0xa1, "optional absent: map(1)");
        let decoded: WithOptional = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "roundtrip WithOptional (None)");
    }

    // Test 4: Optional field present
    {
        let val = WithOptional { name: 42, interval: Some(1000) };
        let len = encode_to_buf(&val, &mut buf);
        assert(buf[0] == 0xa2, "optional present: map(2)");
        let decoded: WithOptional = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "roundtrip WithOptional (Some)");
    }

    // Test 5: Decode reordered keys
    {
        let buf_len = buf.len();
        let mut e = Encoder::new(&mut buf[..]);
        e.map(3).unwrap()
         .str("active").unwrap().bool(false).unwrap()
         .str("humidity").unwrap().u8(99).unwrap()
         .str("temperature").unwrap().i16(-10).unwrap();
        let len = buf_len - e.writer().len();
        let decoded: SensorData = minicbor::decode(&buf[..len]).unwrap();
        assert(decoded.temperature == -10, "reorder: temperature");
        assert(decoded.humidity == 99, "reorder: humidity");
        assert(!decoded.active, "reorder: active");
    }

    // Test 6: Unknown keys skipped
    {
        let buf_len = buf.len();
        let mut e = Encoder::new(&mut buf[..]);
        e.map(4).unwrap()
         .str("temperature").unwrap().i16(0).unwrap()
         .str("extra").unwrap().str("ignored").unwrap()
         .str("humidity").unwrap().u8(50).unwrap()
         .str("active").unwrap().bool(true).unwrap();
        let len = buf_len - e.writer().len();
        let decoded: SensorData = minicbor::decode(&buf[..len]).unwrap();
        assert(decoded.temperature == 0, "unknown keys: temperature");
        assert(decoded.humidity == 50, "unknown keys: humidity");
        assert(decoded.active, "unknown keys: active");
    }

    // Test 7: Enum unit variant
    {
        let val = Command::Reset;
        let len = encode_to_buf(&val, &mut buf);
        let mut d = Decoder::new(&buf[..len]);
        assert(d.str().unwrap() == "Reset", "enum unit: wire = \"Reset\"");
        let decoded: Command = minicbor::decode(&buf[..len]).unwrap();
        assert(decoded == Command::Reset, "roundtrip enum unit");
    }

    // Test 8: Enum struct variant
    {
        let val = Command::SetInterval { ms: 500 };
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Command = minicbor::decode(&buf[..len]).unwrap();
        assert(decoded == Command::SetInterval { ms: 500 }, "roundtrip enum struct");
    }

    // Test 9: skip field
    {
        let val = WithSkip { name: 1, cached: 9999, value: 42 };
        let len = encode_to_buf(&val, &mut buf);
        assert(buf[0] == 0xa2, "skip: map(2)");
        let decoded: WithSkip = minicbor::decode(&buf[..len]).unwrap();
        assert(decoded.name == 1, "skip: name");
        assert(decoded.cached == 0, "skip: cached == Default");
        assert(decoded.value == 42, "skip: value");
    }

    // Test 10: array field
    {
        let val = WithArray { label: 5, readings: [100, 200, 300] };
        let len = encode_to_buf(&val, &mut buf);
        let decoded: WithArray = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "roundtrip WithArray");
    }

    // Test 11: empty struct
    {
        let val = Empty {};
        let len = encode_to_buf(&val, &mut buf);
        assert(buf[0] == 0xa0, "empty struct: map(0)");
        assert(len == 1, "empty struct: 1 byte");
        let decoded: Empty = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "roundtrip Empty");
    }

    // Test 12: all optional, all None
    {
        let val = AllOpt { a: None, b: None };
        let len = encode_to_buf(&val, &mut buf);
        assert(buf[0] == 0xa0, "all none: map(0)");
        let decoded: AllOpt = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "roundtrip AllOpt (None)");
    }

    // Test 13: generic struct
    {
        let val: Generic<u16> = Generic { value: 1000, label: 5 };
        let len = encode_to_buf(&val, &mut buf);
        let decoded: Generic<u16> = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "roundtrip Generic<u16>");
    }

    // Test 14: encode buffer too small
    {
        let val = SensorData { temperature: 23, humidity: 65, active: true };
        let len = encode_to_buf(&val, &mut buf);
        let mut small = [0u8; 1];
        assert(minicbor::encode(&val, &mut small[..]).is_err(), "buf too small: err");
        assert(minicbor::encode(&val, &mut buf[..len]).is_ok(), "buf exact: ok");
        if len > 0 {
            assert(minicbor::encode(&val, &mut buf[..len - 1]).is_err(), "buf one short: err");
        }
    }

    // Test 15: decode truncated
    {
        let val = SensorData { temperature: 23, humidity: 65, active: true };
        let len = encode_to_buf(&val, &mut buf);
        let result: Result<SensorData, _> = minicbor::decode(&buf[..len - 1]);
        assert(result.is_err(), "decode truncated: err");
    }

    // Test 16: decode non-string key
    {
        let buf_len = buf.len();
        let mut e = Encoder::new(&mut buf[..]);
        e.map(1).unwrap().u32(0).unwrap().u8(1).unwrap();
        let len = buf_len - e.writer().len();
        let result: Result<SensorData, _> = minicbor::decode(&buf[..len]);
        assert(result.is_err(), "non-string key: err");
    }

    // Test 17: text_keys implies map
    {
        let val = SensorData { temperature: 1, humidity: 2, active: true };
        let len = encode_to_buf(&val, &mut buf);
        assert(buf[0] == 0xa3, "implies map: map(3)");
        let decoded: SensorData = minicbor::decode(&buf[..len]).unwrap();
        assert(val == decoded, "implies map: roundtrip");
    }

    hprintln!("=== ALL TESTS PASSED ===");
    debug::exit(debug::EXIT_SUCCESS);
    loop {}
}
