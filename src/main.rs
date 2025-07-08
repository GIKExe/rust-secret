use std::{
	fs,
	io::{BufWriter, Write},
	panic,
};

use clap::{ArgAction, Parser};
use jpeg_decoder::{ColorTransform, Decoder};
use jpeg_encoder::{ColorType, Encoder};

#[derive(Parser)]
#[command(
	name = "secret",
	version = "1.0",
	about = "hides files in jpeg",
	long_about = None
)]
struct Args {
	/// Input Brainfuck file
	#[arg(short = 'i', long = "input")]
	input: Option<String>,

	/// Output file path
	#[arg(short = 'o', long = "output")]
	output: Option<String>,

	#[arg(short = 'd', long = "data")]
	data: Option<String>,

	/// Emit LLVM IR instead of binary
	#[arg(short = 'r', long = "read", action = ArgAction::SetTrue)]
	read: bool,
}

fn get_z(num: usize) -> String {
	if num >= 1_048_576 {
		format!("{:.3} МБ", (num as f64) / 1_048_576.0)
	} else if num >= 1024 {
		format!("{:.3} КБ", (num as f64) / 1024.0)
	} else {
		format!("{} Б", num)
	}
}

fn write_bits_to_pixels(pixels: &mut [u8], data: &[u8]) {
	let mut data = data.iter();
	let mut sh = 0u8;
	let mut byte = *data.next().unwrap_or(&0);
	for chunk in pixels.chunks_exact_mut(3) {
		// chunk = [Y, Cb, Cr]
		chunk[0] = (chunk[0] & 252) | ((byte >> sh) & 3);
		sh += 2;
		if sh > 7 {
			byte = *data.next().unwrap_or(&0);
			sh = 0
		};
	}
}

fn read_bits_from_pixels(pixels: &[u8]) -> Vec<u8> {
	let mut data = Vec::new();
	let mut sh = 0u8;
	let mut byte = 0;
	for chunk in pixels.chunks_exact(3) {
		// chunk = [Y, Cb, Cr]
		byte |= (chunk[0] & 3) << sh;
		sh += 2;
		if sh > 7 {
			data.push(byte);
			byte = 0;
			sh = 0
		}
	}
	data.push(byte);
	data
}

fn process() {
	let args = Args::parse();

	let path = args.input.unwrap_or("input.jpg".to_string());
	println!("Чтение {}", &path);
	let file = fs::File::open(&path).unwrap_or_else(|_| panic!("Не удалось открыть {}", &path));
	let mut decoder = Decoder::new(file);
	decoder.set_color_transform(ColorTransform::YCbCr);
	let mut pixels = decoder.decode().expect("Не удалось декодировать");
	let metadata = decoder.info().expect("Не удалось получить метаданные");

	if args.read {
		// Чтение файла из фото
		println!("Читаем файл из фото...");
		let data = read_bits_from_pixels(&pixels);

		let path = args.output.unwrap_or("output.jpg".to_owned());
		let mut output_file = fs::File::create(&path)
			.unwrap_or_else(|_| panic!("Не удалось создать/перезаписать файл: {}", &path));
		output_file
			.write_all(&data)
			.expect("Не удалось записать данные в файл");
	} else {
		// Запись файла в фото
		println!("Всего пикселей: {}", pixels.len());
		let max_bytes = pixels.len() * 2 / 8;
		println!("Доступно для записи: {}\n", get_z(max_bytes));

		let path = args.data.unwrap_or("input.7z".to_owned());
		println!("Чтение {}", &path);
		let data = fs::read("input.7z").unwrap_or_else(|_| panic!("Не удалось открыть {}", &path));
		println!("Размер файла: {}", get_z(data.len()));

		if data.len() > max_bytes {
			return println!("Файл превышает доступное место");
		}

		println!("Вшиваем файл в фото...");
		write_bits_to_pixels(&mut pixels, &data);

		println!("Обратное преобразование");
		let path = args.output.unwrap_or("output.jpg".to_owned());
		let file =
			fs::File::create(&path).unwrap_or_else(|_| panic!("Не удалось записать файл: {}", &path));
		let writer = BufWriter::new(file);
		let encoder = Encoder::new(writer, 100);
		encoder
			.encode(&pixels, metadata.width, metadata.height, ColorType::Ycbcr)
			.expect("Ошибка кодирования");

		// let (width, height) = rgb_img.dimensions();
		// let mut output_img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

		// for (x, y, pixel) in output_img.enumerate_pixels_mut() {
		// 	let index = (y * width + x) as usize;
		// 	let (r, g, b) = pixels[index];
		// 	*pixel = Rgb([r, g, b]);
		// }

		// // Сохранение в JPEG
		// let path = args.output.unwrap_or("output.jpg".to_owned());
		// let mut output_file = fs::File::create(&path)
		// 	.unwrap_or_else(|_| panic!("Не удалось записать файл: {}", &path));
		// let mut encoder = JpegEncoder::new_with_quality(&mut output_file, 100);
		// encoder
		// 	.encode_image(&output_img)
		// 	.expect("Не удалось закодировать изображение");
	}
}

fn main() {
	let result = panic::catch_unwind(|| {
		process();
	});
	match result {
		Ok(_) => println!("Программа завершилась без ошибок\n\n "),
		Err(e) => println!("{e:?}"),
	}
}
