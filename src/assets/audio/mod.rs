mod params;
pub use self::params::AudioImportParams;

use std::fs::{self, File};
use std::io::{Read, Write};

use crayon_audio::assets::clip_loader;

use vorbis;

use assets::{AssetImporter, AssetParams, ResourceType};
use platform::Compression;
use workspace::database::{AssetIntermediateGenerator, AssetMetadataGenerator};

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

pub struct AudioImporter {}

impl AssetImporter for AudioImporter {
    fn compile(&self, db: &mut AssetIntermediateGenerator) -> Result<()> {
        if !db.modified() && !db.intermediate_modified("clip") {
            return Ok(());
        }

        let in_file = File::open(db.path()).unwrap();

        let (data, channels, rate) = match db.name().extension().unwrap().to_str().unwrap() {
            "mp3" => mp3(in_file),
            "wav" => wav(in_file),
            "flac" => flac(in_file),
            "ogg" => vorbis(in_file),
            v => bail!("{} is not supported yet!", v),
        };

        let bps = channels as u32 * rate as u32;

        info!(
            "Compiles audio flie {}. (Channels: {}, SampleRate: {}, Len: {:.2}s, Size: {})",
            db.name().display(),
            channels,
            rate,
            f64::from(data.len() as u32) / f64::from(bps),
            data.len()
        );

        let params: AudioImportParams = db.params().into();
        let mut encoder = vorbis::Encoder::new(channels, rate, params.compression.into())?;

        let mut out_file = File::create(&db.intermediate("clip", true))?;
        out_file.write(encoder.encode(&data)?.as_slice())?;
        out_file.write(encoder.flush()?.as_slice())?;

        Ok(())
    }

    fn compile_metadata(&self, db: &mut AssetMetadataGenerator) -> Result<()> {
        let name = db.name().to_owned();
        db.add(&name, ResourceType::AudioClip);

        match db.params() {
            AssetParams::Audio(_) => {}
            _ => db.update_params(AssetParams::Audio(AudioImportParams::default())),
        }

        Ok(())
    }

    fn import(&self, db: &mut AssetIntermediateGenerator) -> Result<()> {
        let name = db.name().to_owned();
        if !db.intermediate_modified("clip") && !db.resource_modified(&name) {
            return Ok(());
        }

        info!("Imports audio clip {}.", db.name().display());

        let mut in_file = File::open(db.intermediate("clip", false))?;
        let mut buf = Vec::new();
        in_file.read_to_end(&mut buf)?;

        let mut file = fs::File::create(db.resource(&name, true))?;
        file.write_all(&clip_loader::MAGIC)?;
        file.write_all(&buf)?;

        Ok(())
    }
}

impl From<Compression> for vorbis::VorbisQuality {
    fn from(compression: Compression) -> vorbis::VorbisQuality {
        match compression {
            Compression::None => vorbis::VorbisQuality::HighQuality,
            Compression::HighQuality => vorbis::VorbisQuality::Midium,
            Compression::LowQuality => vorbis::VorbisQuality::HighPerforamnce,
        }
    }
}

fn mp3(file: File) -> (Vec<i16>, u8, u64) {
    use minimp3;

    let mut data = Vec::new();
    let mut channels = 0;
    let mut rate = 0;

    let mut decoder = minimp3::Decoder::new(file);

    while let Some(frame) = decoder.next_frame().ok() {
        channels = frame.channels;
        rate = frame.sample_rate;
        data.extend(frame.data);
    }

    (data, channels as u8, rate as u64)
}

fn wav(file: File) -> (Vec<i16>, u8, u64) {
    use hound;

    let mut reader = hound::WavReader::new(file).unwrap();
    let spec = reader.spec();

    let mut data = Vec::new();

    for v in reader.samples() {
        let v = v.unwrap();
        data.push(v);
    }

    (data, spec.channels as u8, spec.sample_rate as u64)
}

fn vorbis(file: File) -> (Vec<i16>, u8, u64) {
    let mut data = Vec::new();
    let mut channels = 0;
    let mut rate = 0;
    let mut decoder = vorbis::Decoder::new(file).unwrap();
    let packets = decoder.packets();
    for p in packets {
        match p {
            Ok(packet) => {
                channels = packet.channels as u8;
                rate = packet.rate;
                data.extend(&packet.data);
            }
            _ => {}
        }
    }

    (data, channels, rate)
}

fn flac(file: File) -> (Vec<i16>, u8, u64) {
    use claxon;

    let mut reader = claxon::FlacReader::new(file).unwrap();
    let channels = reader.streaminfo().channels as u8;
    let rate = reader.streaminfo().sample_rate as u64;

    let bits_per_sample = reader.streaminfo().bits_per_sample;
    let mut data = Vec::new();
    data.extend(reader.samples().map(|v| {
        let v = v.unwrap();
        if bits_per_sample == 16 {
            v as i16
        } else if bits_per_sample < 16 {
            (v << (16 - bits_per_sample)) as i16
        } else {
            (v >> (bits_per_sample - 16)) as i16
        }
    }));

    (data, channels, rate)
}
