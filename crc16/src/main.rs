use anyhow::{Context, Result};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

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

#[rustfmt::skip]
const MSB: u32     = 0x80000000; // Most significant bit
const CRCPOLY: u32 = 0x80050000; // x^16 + x^15 + x^2 + 1 (shifted << 16)

#[rustfmt::skip]
#[allow(dead_code)]
// Pre-generated table
const TABLE: [u16; 256] = [
        0, 32773, 32783,    10, 32795,    30,    20, 32785, 32819,    54,    60, 32825,    40, 32813, 32807,    34,
    32867,   102,   108, 32873,   120, 32893, 32887,   114,    80, 32853, 32863,    90, 32843,    78,    68, 32833,
    32963,   198,   204, 32969,   216, 32989, 32983,   210,   240, 33013, 33023,   250, 33003,   238,   228, 32993,
      160, 32933, 32943,   170, 32955,   190,   180, 32945, 32915,   150,   156, 32921,   136, 32909, 32903,   130,
    33155,   390,   396, 33161,   408, 33181, 33175,   402,   432, 33205, 33215,   442, 33195,   430,   420, 33185,
      480, 33253, 33263,   490, 33275,   510,   500, 33265, 33235,   470,   476, 33241,   456, 33229, 33223,   450,
      320, 33093, 33103,   330, 33115,   350,   340, 33105, 33139,   374,   380, 33145,   360, 33133, 33127,   354,
    33059,   294,   300, 33065,   312, 33085, 33079,   306,   272, 33045, 33055,   282, 33035,   270,   260, 33025,
    33539,   774,   780, 33545,   792, 33565, 33559,   786,   816, 33589, 33599,   826, 33579,   814,   804, 33569,
      864, 33637, 33647,   874, 33659,   894,   884, 33649, 33619,   854,   860, 33625,   840, 33613, 33607,   834,
      960, 33733, 33743,   970, 33755,   990,   980, 33745, 33779,  1014,  1020, 33785,  1000, 33773, 33767,   994,
    33699,   934,   940, 33705,   952, 33725, 33719,   946,   912, 33685, 33695,   922, 33675,   910,   900, 33665,
      640, 33413, 33423,   650, 33435,   670,   660, 33425, 33459,   694,   700, 33465,   680, 33453, 33447,   674,
    33507,   742,   748, 33513,   760, 33533, 33527,   754,   720, 33493, 33503,   730, 33483,   718,   708, 33473,
    33347,   582,   588, 33353,   600, 33373, 33367,   594,   624, 33397, 33407,   634, 33387,   622,   612, 33377,
      544, 33317, 33327,   554, 33339,   574,   564, 33329, 33299,   534,   540, 33305,   520, 33293, 33287,   514,
];

// Returns the CRC-16 TABLE
const fn crc16_table() -> [u16; 256] {
    let mut table = [0u16; 256];
    let mut i = 0;
    while i < table.len() {
        table[i] = _crc16_byte(i as u8);
        i += 1;
    }
    table
}

// TABLE generator using 2 const functions
const fn _crc16_byte(byte: u8) -> u16 {
    let mut value: u16 = (byte as u16) << 8;
    let mut i = 0;
    while i < 8 {
        value = (value << 1) ^ (((value >> 15) & 1) * (CRCPOLY >> 16) as u16);
        i += 1;
    }
    value
}

// TABLE generator (basic)
#[allow(dead_code)]
fn crc16_table_generator() -> Vec<u16> {
    (0..256).map(|i| crc16_algo(&[i as u8])).collect()
}

// The implementation I first learned
fn crc16_algo(msg: &[u8]) -> u16 {
    // Message must have length > 0
    assert!(!msg.is_empty(), "Invalid input");

    // Handle single byte inputs
    if msg.len() == 1 {
        return crc16_algo(&[0, msg[0]]);
    }

    let mut crc: u32;

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

// The algorithm using a table lookup
fn crc16(msg: &[u8]) -> u16 {
    // Message must have length > 0
    assert!(!msg.is_empty(), "Invalid input");

    // Get the table from our const function
    let table = crc16_table();
    //let table = TABLE;

    let mut crc = 0u16;
    for byte in msg.iter() {
        crc ^= (*byte as u16) << 8;
        crc = (crc << 8) ^ table[(crc >> 8) as usize];
    }
    crc
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

        // Output CRC-16/UMTS
        println!("{input_name}: {}", crc16(&msg));
        //println!("table = {:?}", crc16_table());
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
        crc16_algo(&[]);
    }

    #[test]
    #[should_panic]
    fn test_bad_input_2() {
        crc16(&[]);
    }

    #[test]
    fn table_test() {
        let table = crc16_table();
        assert_eq!(TABLE, table);

        for (i, val) in crc16_table_generator().into_iter().enumerate() {
            assert_eq!(TABLE[i], val);
            assert_eq!(table[i], val);
        }

        for i in 0..256 {
            let data = [i as u8];
            assert_eq!(TABLE[i], crc16(&data));
            assert_eq!(TABLE[i], crc16_algo(&data));
            assert_eq!(TABLE[i], CRC_16_UMTS.checksum(&data));
        }
    }

    #[test]
    fn test1() {
        let data = [5, 0, 255, 255, 255, 255, 0, 0, 0, 0, 2, 0, 1, 1, 0, 0];
        assert_eq!(35273, crc16(&data));
        assert_eq!(crc16(&data), crc16_algo(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));
    }

    #[test]
    fn test2() {
        let data = [
            170, 170, 170, 170, 170, 170, 170, 170, 204, 204, 204, 204, 204, 204, 204, 204,
        ];
        assert_eq!(43036, crc16(&data));
        assert_eq!(crc16(&data), crc16_algo(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));
    }

    #[test]
    fn test3() {
        let data = [100, 97, 118, 101];
        assert_eq!(25309, crc16(&data));
        assert_eq!(crc16(&data), crc16_algo(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));

        assert_eq!(25309, crc16(b"dave"));
        assert_eq!(crc16(b"dave"), crc16_algo(b"dave"));
        assert_eq!(crc16(b"dave"), CRC_16_UMTS.checksum(b"dave"));
    }

    #[test]
    fn test4() {
        let data = [49, 50, 51, 52, 53, 54, 55, 56, 57];
        assert_eq!(65256, crc16(&data));
        assert_eq!(crc16(&data), crc16_algo(&data));
        assert_eq!(crc16(&data), CRC_16_UMTS.checksum(&data));

        assert_eq!(65256, crc16(b"123456789"));
        assert_eq!(crc16(b"123456789"), crc16_algo(b"123456789"));
        assert_eq!(crc16(b"123456789"), CRC_16_UMTS.checksum(b"123456789"));
    }

    #[test]
    fn crc_module_constants() {
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
