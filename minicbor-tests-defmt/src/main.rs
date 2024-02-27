#![no_std]
#![no_main]

use minicbor;

use defmt;

use defmt_rtt as _;
use panic_halt as _;
use cortex_m as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let buf = &[0xff];
    let mut decoder = minicbor::decode::Decoder::new(buf);
    defmt::error!("Problem is {}", decoder.bool().unwrap_err());

    let buf = &[0x63];
    let mut decoder = minicbor::decode::Decoder::new(buf);
    defmt::error!("Problem is {}", decoder.str().unwrap_err());

    defmt::panic!("Done");
}
