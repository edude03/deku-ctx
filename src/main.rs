use deku::{DekuRead, DekuReader, DekuWrite, DekuWriter};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
// #[deku(endian = "big")]
pub struct VarInt(u16); // Not really, but for demonstration sake

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct CompactString {
    length: VarInt,
    #[deku(count = "length.0")] // Not exactly because VarInt is nullable in reality but anyway
    value: Vec<u8>,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]

pub struct CompactArray<T>
where
    T: for<'a> DekuReader<'a, deku::ctx::Endian>
        + DekuWriter<deku::ctx::Endian>
        + PartialEq
        + std::fmt::Debug,
{
    length: VarInt,
    #[deku(count = "length.0")]
    content: Vec<T>,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct Topic {
    name: CompactString,
}

#[derive(Debug, PartialEq, DekuRead)]
#[deku(endian = "big")]
pub struct Request {
    topics: CompactArray<Topic>,
}

fn main() {}

#[cfg(test)]
mod test {
    use deku::DekuContainerRead;

    use super::*;

    #[test]
    fn basic() {
        let input: Vec<u8> = vec![
            0x00, 0x01, // 1 topic
            0x00, 0x03, // compactstring with 3 chars
            0x66, 0x6f, 0x6f, // foo
        ];

        let expected = Request {
            topics: CompactArray {
                length: VarInt(1),
                content: vec![Topic {
                    name: CompactString {
                        length: VarInt(3),
                        value: vec![0x66, 0x6f, 0x6f],
                    },
                }],
            },
        };

        let (_, actual) = Request::from_bytes((&input, 0)).unwrap();
        assert_eq!(expected, actual)
    }
}
