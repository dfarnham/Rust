use anyhow::{Context, Result};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

#[rustfmt::skip]
const MSB: u32     = 0x80000000; // Most significant bit
const CRCPOLY: u32 = 0x80050000; // x^16 + x^15 + x^2 + 1 (shifted << 16)

// CRC-16, CRC-16/UMTS
//
// https://reveng.sourceforge.io/crc-catalogue/all.htm
// width=16 poly=0x8005
// init=0x0000
// refin=false
// refout=false
// xorout=0x0000
// check=0xfee8
// residue=0x0000
// name="CRC-16/UMTS"
fn crc16(msg: &[u8]) -> u16 {
    let mut crc;

    // Minimum input is 2 bytes
    assert!(msg.len() > 1, "Invalid input");

    // Load the first 2-bytes
    crc = (msg[0] as u32) << 24;
    crc |= (msg[1] as u32) << 16;

    // The input is 2-byte zero padded
    for byte in msg.iter().skip(2).chain([0, 0].iter()) {
        crc ^= (*byte as u32) << 8;
        for _ in 0..8 {
            if crc & MSB == 0 {
                crc <<= 1;
            } else {
                crc = (crc << 1) ^ CRCPOLY;
            }
        }
    }
    ((crc >> 16) & 0x0000ffff) as u16
}

fn main() -> Result<(), Box<dyn Error>> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about=None)]
    struct Args {
        /// file|stdin, filename of "-" implies stdin
        files: Vec<std::path::PathBuf>,
    }
    let args = Args::parse();

    let files = match args.files.is_empty() {
        true => vec![std::path::PathBuf::from("-")],
        false => args.files,
    };

    for file in files {
        // Receive data from stdin|file, note a filename of "-" implies stdin
        let mut msg = vec![];
        let input_name: String = match file.as_os_str() != "-" {
            true => {
                File::open(&file)
                    .with_context(|| format!("could not open file `{:?}`", file.as_os_str()))?
                    .read_to_end(&mut msg)
                    .with_context(|| format!("could not read file `{:?}`", file.as_os_str()))?;
                file.to_string_lossy().into()
            }
            false => {
                io::stdin()
                    .read_to_end(&mut msg)
                    .with_context(|| "could not read `stdin`")?;
                "<stdin>".into()
            }
        };

        // Add 2-bytes of zero padding
        println!("{input_name}: {}", crc16(&msg));
    }

    Ok(())
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    const CRC_16_ARC               : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_ARC);
    const CRC_16_CDMA2000          : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_CDMA2000);
    const CRC_16_CMS               : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_CMS);
    const CRC_16_DDS_110           : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_DDS_110);
    const CRC_16_DECT_R            : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_DECT_R);
    const CRC_16_DECT_X            : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_DECT_X);
    const CRC_16_DNP               : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_DNP);
    const CRC_16_EN_13757          : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_EN_13757);
    const CRC_16_GENIBUS           : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_GENIBUS);
    const CRC_16_GSM               : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_GSM);
    const CRC_16_IBM_3740          : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_3740);
    const CRC_16_IBM_SDLC          : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
    const CRC_16_ISO_IEC_14443_3_A : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_ISO_IEC_14443_3_A);
    const CRC_16_KERMIT            : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_KERMIT);
    const CRC_16_LJ1200            : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_LJ1200);
    const CRC_16_MAXIM_DOW         : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_MAXIM_DOW);
    const CRC_16_MCRF4XX           : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_MCRF4XX);
    const CRC_16_MODBUS            : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_MODBUS);
    const CRC_16_NRSC_5            : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_NRSC_5);
    const CRC_16_OPENSAFETY_A      : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_OPENSAFETY_A);
    const CRC_16_OPENSAFETY_B      : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_OPENSAFETY_B);
    const CRC_16_PROFIBUS          : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_PROFIBUS);
    const CRC_16_RIELLO            : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_RIELLO);
    const CRC_16_SPI_FUJITSU       : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_SPI_FUJITSU);
    const CRC_16_T10_DIF           : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_T10_DIF);
    const CRC_16_TELEDISK          : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_TELEDISK);
    const CRC_16_TMS37157          : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_TMS37157);
    const CRC_16_UMTS              : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_UMTS);
    const CRC_16_USB               : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_USB);
    const CRC_16_XMODEM            : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);

    #[test]
    #[should_panic]
    fn test_bad_input_1() {
        crc16(&[]);
    }

    #[test]
    #[should_panic]
    fn test_bad_input_2() {
        crc16(&[1]);
    }

    #[test]
    fn test_zero() {
        let data = [0, 0];
        assert_eq!(0, crc16(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));
    }

    #[test]
    fn test1() {
        let data = [5, 0, 255, 255, 255, 255, 0, 0, 0, 0, 2, 0, 1, 1, 0, 0];
        assert_eq!(35273, crc16(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));
    }

    #[test]
    fn test2() {
        let data = [
            170, 170, 170, 170, 170, 170, 170, 170, 204, 204, 204, 204, 204, 204, 204, 204,
        ];
        assert_eq!(43036, crc16(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));
    }

    #[test]
    fn test3() {
        let data = [100, 97, 118, 101];
        assert_eq!(25309, crc16(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));

        assert_eq!(25309, crc16(b"dave"));
        assert_eq!(crc16(b"dave"), CRC_16_UMTS.checksum(b"dave"));
    }

    #[test]
    fn test4() {
        let data = [49, 50, 51, 52, 53, 54, 55, 56, 57];
        assert_eq!(65256, crc16(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));

        assert_eq!(65256, crc16(b"123456789"));
        assert_eq!(crc16(b"123456789"), CRC_16_UMTS.checksum(b"123456789"));
    }

    #[test]
    fn crc_constants() {
        let data = b"123456789";
        assert_eq!(47933 , CRC_16_ARC.checksum(data));
        assert_eq!(19462 , CRC_16_CDMA2000.checksum(data));
        assert_eq!(44775 , CRC_16_CMS.checksum(data));
        assert_eq!(40655 , CRC_16_DDS_110.checksum(data));
        assert_eq!(126   , CRC_16_DECT_R.checksum(data));
        assert_eq!(127   , CRC_16_DECT_X.checksum(data));
        assert_eq!(60034 , CRC_16_DNP.checksum(data));
        assert_eq!(49847 , CRC_16_EN_13757.checksum(data));
        assert_eq!(54862 , CRC_16_GENIBUS.checksum(data));
        assert_eq!(52796 , CRC_16_GSM.checksum(data));
        assert_eq!(10673 , CRC_16_IBM_3740.checksum(data));
        assert_eq!(36974 , CRC_16_IBM_SDLC.checksum(data));
        assert_eq!(48901 , CRC_16_ISO_IEC_14443_3_A.checksum(data));
        assert_eq!(8585  , CRC_16_KERMIT.checksum(data));
        assert_eq!(48628 , CRC_16_LJ1200.checksum(data));
        assert_eq!(17602 , CRC_16_MAXIM_DOW.checksum(data));
        assert_eq!(28561 , CRC_16_MCRF4XX.checksum(data));
        assert_eq!(19255 , CRC_16_MODBUS.checksum(data));
        assert_eq!(41062 , CRC_16_NRSC_5.checksum(data));
        assert_eq!(23864 , CRC_16_OPENSAFETY_A.checksum(data));
        assert_eq!(8446  , CRC_16_OPENSAFETY_B.checksum(data));
        assert_eq!(43033 , CRC_16_PROFIBUS.checksum(data));
        assert_eq!(25552 , CRC_16_RIELLO.checksum(data));
        assert_eq!(58828 , CRC_16_SPI_FUJITSU.checksum(data));
        assert_eq!(53467 , CRC_16_T10_DIF.checksum(data));
        assert_eq!(4019  , CRC_16_TELEDISK.checksum(data));
        assert_eq!(9905  , CRC_16_TMS37157.checksum(data));
        assert_eq!(65256 , CRC_16_UMTS.checksum(data));
        assert_eq!(46280 , CRC_16_USB.checksum(data));
        assert_eq!(12739 , CRC_16_XMODEM.checksum(data));
    }
}
