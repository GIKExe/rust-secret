use std::{fs::File, io::BufWriter};
use png::{Decoder, Encoder, OutputInfo};

pub fn read_image(path: impl AsRef<str>) -> (Vec<u8>, OutputInfo) {
	let file = File::open(path.as_ref()).unwrap();
	let decoder = Decoder::new(file);
	let mut reader = decoder.read_info().unwrap();

	let mut pixels = vec![0; reader.output_buffer_size()];
	let info = reader.next_frame(&mut pixels).unwrap();

	pixels.truncate(info.buffer_size());
	(pixels, info)
}

pub fn write_image(path: impl AsRef<str>, pixels: &[u8], info: OutputInfo) {
	let file = File::create(path.as_ref()).unwrap();
	let writer = BufWriter::new(file);

	let mut encoder = Encoder::new(writer, info.width, info.height);
	// Копируем параметры из исходного изображения
	encoder.set_color(info.color_type);
	encoder.set_depth(info.bit_depth);

	let mut writer = encoder.write_header().unwrap();
	writer.write_image_data(pixels).unwrap();
}