use std::{fs::File, io::BufWriter};
use png::{Decoder, Encoder, OutputInfo};

use crate::error::Error;

pub fn read_image(path: impl AsRef<str>) -> Result<(Vec<u8>, OutputInfo), Error> {
	let path = path.as_ref();
	let file = File::open(path).map_err(|_| Error::FileRead(path.to_string()))?;
	let decoder = Decoder::new(file);
	let mut reader = decoder.read_info().map_err(|e| Error::Decoder(e.to_string()))?;

	let mut pixels = vec![0; reader.output_buffer_size()];
	let info = reader.next_frame(&mut pixels).map_err(|e| Error::Decoder(e.to_string()))?;

	pixels.truncate(info.buffer_size());
	Ok((pixels, info))
}

pub fn write_image(path: impl AsRef<str>, pixels: &[u8], info: OutputInfo) -> Result<(), Error>{
	let path = path.as_ref();
	let file = File::create(path).map_err(|_| Error::FileWrite(path.to_string()))?;
	let writer = BufWriter::new(file);

	let mut encoder = Encoder::new(writer, info.width, info.height);
	// Копируем параметры из исходного изображения
	encoder.set_color(info.color_type);
	encoder.set_depth(info.bit_depth);

	let mut writer = encoder.write_header().map_err(|e| Error::Encoder(e.to_string()))?;
	writer.write_image_data(pixels).map_err(|e| Error::Encoder(e.to_string()))?;
	Ok(())
}