use io::sha;
use gfx;

use core::fmt::Write;

static INPUTS: [&[u8]; 7] = [
    b"",
    b"a",
    b"ab",
    b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    b"aaaaaaaaaaaggGaaaaaaggaaaaaaaagaaaaaaaaaaaaGgaaaaaaaaagaaaaaaaaa",
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-+=",
    b"Call me Ishmael. Some years ago - never mind how long precisely - having \
        little or no money in my purse, and nothing particular to interest me on \
        shore, I thought I would sail about a little and see the watery part of the \
        world. It is a way I have of driving off the spleen and regulating the \
        circulation. Whenever I find myself growing grim about the mouth; whenever \
        it is a damp, drizzly November in my soul; whenever I find myself involuntarily \
        pausing before coffin warehouses, and bringing up the rear of every funeral I \
        meet; and especially whenever my hypos get such an upper hand of me, that it \
        requires a strong moral principle to prevent me from deliberately stepping into \
        the street, and methodically knocking people's hats off - then, I account it \
        high time to get to sea as soon as I can. This is my substitute for pistol and ball.",
];

static SUMS_160: [&[u8]; 7] = [
    b"DA39A3EE5E6B4B0D3255BFEF95601890AFD80709",
    b"86F7E437FAA5A7FCE15D1DDCB9EAEAEA377667B8",
    b"DA23614E02469A0D7C7BD1BDAB5C9C474B1904DC",
    b"0098BA824B5C16427BD7A1122A5A442A25EC644D",
    b"5635174F74D73920A19E7736B30100CD736B9402",
    b"B56130BF506630D047DABFCA15E6DE966385DD77",
    b"5969D0ADB971C33CEF609D82B3E5E40B23ACDA17",
];

static SUMS_224: [&[u8]; 7] = [
    b"D14A028C2A3A2BC9476102BB288234C415A2B01F828EA62AC5B3E42F",
    b"ABD37534C7D9A2EFB9465DE931CD7055FFDB8879563AE98078D6D6D5",
    b"DB3CDA86D4429A1D39C148989566B38F7BDA0156296BD364BA2F878B",
    b"A88CD5CDE6D6FE9136A4E58B49167461EA95D388CA2BDB7AFDC3CBF4",
    b"408DD6E1AB4F566392E9ED17A0ECAB0BB99BE9C06C4DC65FBA274336",
    b"180801D0C38AA714D3C04BE1DBA000873655B719B6EB116D114355F0",
    b"615847579C9C5332A6B779A107C384EDA18F59F9CC0390FBE7708E00",
];

static SUMS_256: [&[u8]; 7] = [
    b"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
    b"CA978112CA1BBDCAFAC231B39A23DC4DA786EFF8147C4E72B9807785AFEE48BB",
    b"FB8E20FC2E4C3F248C60C39BD652F3C1347298BB977B8B4D5903B85055620603",
    b"FFE054FE7AE0CB6DC65C3AF9B61D5209F439851DB43D0BA5997337DF154668EB",
    b"52CBACF2286644C497E14D5A147F245636376458FAF0406A636B8678BEAE14CC",
    b"C3AEB0F036DFB5896CCD9AC7B82E095228E4EB0CFED15D928F7956D52976FE2D",
    b"60F7C90FAACA7BC77A1ADCF2565F784790A491B4F8AF14798232D2FB073EE6A8",
];

fn match_hexstr(bytes: &[u8], hex_str: &[u8]) -> bool {
    if hex_str.len() != bytes.len() * 2 {
        write!(gfx::LogWriter, "Size mismatch: hex {} bytes vs data {} bytes", hex_str.len(), bytes.len() * 2);
        return false; // Sizes don't match
    }

    let mut byte_it = bytes.iter();
    let mut hex_it = hex_str.iter();
    while let (Some(src_byte), Some(top_n), Some(bot_n)) = (byte_it.next(), hex_it.next(), hex_it.next()) {
        let to_num = |letter: u8| {
            match letter {
                b'A'...b'F' => letter - b'A' + 10,
                b'a'...b'f' => letter - b'a' + 10,
                b'0'...b'9' => letter - b'0',
                _ => panic!("Attempted to match invalid hex string!")
            }
        };
        let hex_byte = to_num(*top_n) << 4 | to_num(*bot_n);
        if *src_byte != hex_byte {
            write!(gfx::LogWriter, "Failed, byte {:02X} vs hex {}{}", *src_byte, *top_n as char, *bot_n as char);
            return false;
        }
    }
    true
}

fn test_hashes<OUT>(name: &str, func: fn(&[u8]) -> OUT, sums: &[&[u8]])
        where OUT: AsRef<[u8]> {
    write!(gfx::LogWriter, "Testing {} hashes...", name);
    for (input, hashstr) in INPUTS.iter().zip(sums.iter()) {
        let hash = func(input);
        if match_hexstr(hash.as_ref(), hashstr) {
            gfx::log(b"SUCCESS ");
        } else {
            gfx::log(b"FAILURE ");
        }
    }
    gfx::log(b"!\n");
}

pub fn main() {
    use core::fmt::Write;
    let mut logger = gfx::LogWriter;
    gfx::clear_screen(0xFF, 0xFF, 0xFF);

    test_hashes("SHA1", sha::hash_160, &SUMS_160);
    test_hashes("SHA224", sha::hash_224, &SUMS_224);
    test_hashes("SHA256", sha::hash_256, &SUMS_256);
}
