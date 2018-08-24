use std::io::{Read, Seek, SeekFrom};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};

const FILE_IDENTIFIER: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ktx {
    pub gl_type: u32,
    pub gl_type_size: u32,
    pub gl_format: u32,
    pub gl_internal_format: u32,
    pub gl_base_internal_format: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixel_depth: u32,
    pub number_of_array_elements: u32,
    pub number_of_faces: u32,
    pub number_of_mipmap_levels: u32,
    pub bytes_of_key_value_data: u32,
    pub textures: Vec<Box<[u8]>>,
}

impl Ktx {
    pub fn parse<R: Read + Seek>(source: &mut R) -> Result<Ktx> {
        // Read identifier
        let mut buffer: [u8; 12] = [0; 12];
        source.read_exact(&mut buffer)?;
        if buffer != FILE_IDENTIFIER {
            bail!("File is not a KTX texture container. FILE_IDENTIFIER not matches.");
        }

        // Read endianness
        let mut buffer: [u8; 4] = [0; 4];
        source.read_exact(&mut buffer)?;
        let little_endian: bool = match buffer[0] {
            0x01 => true,
            0x04 => false,
            _ => bail!("File is not a KTX texture container."),
        };

        if little_endian {
            Ktx::deserialize::<R, LittleEndian>(source)
        } else {
            Ktx::deserialize::<R, BigEndian>(source)
        }
    }

    fn deserialize<R: Read + Seek, Order: ByteOrder>(source: &mut R) -> Result<Ktx> {
        let mut ktx = Ktx {
            gl_type: source.read_u32::<Order>()?,
            gl_type_size: source.read_u32::<Order>()?,
            gl_format: source.read_u32::<Order>()?,
            gl_internal_format: source.read_u32::<Order>()?,
            gl_base_internal_format: source.read_u32::<Order>()?,
            pixel_width: source.read_u32::<Order>()?,
            pixel_height: source.read_u32::<Order>()?,
            pixel_depth: source.read_u32::<Order>()?,
            number_of_array_elements: source.read_u32::<Order>()?,
            number_of_faces: source.read_u32::<Order>()?,
            number_of_mipmap_levels: source.read_u32::<Order>()?,
            bytes_of_key_value_data: source.read_u32::<Order>()?,
            textures: Vec::new(),
        };

        if ktx.number_of_array_elements != 0 {
            bail!("Array texture is not supported.");
        }

        if ktx.number_of_faces != 1 {
            bail!("Cube texture is not supported.");
        }

        if ktx.pixel_depth != 0 {
            bail!("Multi-dimensions texture is not supported.");
        }

        // If number_of_mipmap_levels equals 0, it indicates that a full mipmap pyramid should be
        // generated from level 0 at load time (this is usually not allowed for compressed formats).
        if ktx.number_of_mipmap_levels == 0 {
            ktx.number_of_mipmap_levels = 1;
        }

        source.seek(SeekFrom::Current(ktx.bytes_of_key_value_data as i64))?;

        for _ in 0..ktx.number_of_mipmap_levels {
            let size = source.read_u32::<Order>()?;

            let mut buf = Vec::with_capacity(size as usize);
            unsafe { buf.set_len(size as usize) };

            source.read_exact(buf.as_mut_slice())?;
            source.seek(SeekFrom::Current(3 - ((size as i64 + 3) % 4)))?;

            ktx.textures.push(buf.into_boxed_slice());
        }

        Ok(ktx)
    }
}
