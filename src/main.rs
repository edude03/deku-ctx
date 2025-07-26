use deku::{
    ctx::Endian, no_std_io, writer::Writer, DekuError, DekuRead, DekuReader, DekuWrite, DekuWriter,
};

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

#[derive(Debug, PartialEq)]
pub struct CompactArray<T> {
    length: VarInt,
    content: Vec<T>,
}

impl<'a, T> DekuReader<'a, Endian> for CompactArray<T>
where
    T: for<'b> DekuReader<'b, Endian>,
{
    fn from_reader_with_ctx<R: deku::no_std_io::Read + no_std_io::Seek>(
        reader: &mut deku::reader::Reader<R>,
        endian: deku::ctx::Endian,
    ) -> Result<Self, DekuError>
    where
        Self: Sized,
    {
        let length = <VarInt as DekuReader<'_, _>>::from_reader_with_ctx(reader, endian)?;
        let limit = deku::ctx::Limit::new_count(length.0 as usize);

        let content =
            <Vec<T> as DekuReader<'_, _>>::from_reader_with_ctx(reader, (limit, (endian)))?;

        Ok(CompactArray { length, content })
    }
}

impl<T> DekuWriter<Endian> for CompactArray<T>
where
    T: DekuWriter<Endian>,
{
    #[doc = " Write type to bytes"]
    fn to_writer<W: no_std_io::Write + no_std_io::Seek>(
        &self,
        writer: &mut Writer<W>,
        ctx: Endian,
    ) -> Result<(), DekuError> {
        self.content.to_writer(writer, ctx)?;
        self.content.to_writer(writer, ctx)?;

        Ok(())
    }
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
