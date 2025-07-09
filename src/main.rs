use std::{fs, io::Write, panic};
use clap::{ArgAction, Parser};
use image::{ImageReader, Rgb};

#[derive(Parser)]
#[command(
	name = "secret",
	version = "1.0",
	about = "hides files in jpeg",
	long_about = None
)]
struct Args {
	/// Путь входного файла
	input: String,

	/// Путь выходного файла
	output: String,

	/// Путь вшиваемого файла, активирует запись
	#[arg(short = 'd', long = "data")]
	data: Option<String>,
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

type Pixel = (u8, u8, u8);

fn write_bits_to_pixels(pixels: &mut Vec<Pixel>, data: &[u8]) {
	let mut data = data.iter();
	let mut sh = 0u8;
	let mut byte = *data.next().unwrap_or(&0);
	for (r, g, b) in pixels {
		*r = (*r & 252) | ((byte >> sh) & 3); sh += 2;
		if sh > 7 { byte = *data.next().unwrap_or(&0); sh = 0 };

		*g = (*g & 252) | ((byte >> sh) & 3); sh += 2;
		if sh > 7 { byte = *data.next().unwrap_or(&0); sh = 0 };

		*b = (*b & 252) | ((byte >> sh) & 3); sh += 2;
		if sh > 7 { byte = *data.next().unwrap_or(&0); sh = 0 };
	}
}

fn read_bits_from_pixels(pixels: &Vec<Pixel>) -> Vec<u8> {
	let mut data = Vec::new();
	let mut sh = 0u8;
	let mut byte = 0;
	for (r, g, b) in pixels {
		byte |= (*r & 3) << sh; sh += 2;
		if sh > 7 { data.push(byte); byte = 0; sh = 0 }

		byte |= (*g & 3) << sh; sh += 2;
		if sh > 7 { data.push(byte); byte = 0; sh = 0 }

		byte |= (*b & 3) << sh; sh += 2;
		if sh > 7 { data.push(byte); byte = 0; sh = 0 }
	}
	data.push(byte);
	data
}

fn process() {
	let args = Args::parse();

	let path = args.input.clone();
	println!("Чтение {}", &path);
	let mut image = ImageReader::open(&path)
		.unwrap_or_else(|_| panic!("Не удалось открыть {}", &path))
		.decode()
		.expect("Не удалось декодировать изображение")
		.into_rgb8();

	let mut pixels: Vec<(u8, u8, u8)> = image.pixels()
		.map(|p| (p[0], p[1], p[2])).collect();

	if args.data.is_none() {
		println!("Читаем файл из фото...");
		let mut data = read_bits_from_pixels(&pixels);
		if let Some(index) = data.iter().rposition(|&b| b != 0x00) {
			data.truncate(index + 1); // Сохраняем последний ненулевой элемент
		}

		let path = args.output.clone();
		fs::File::create(&path)
			.unwrap_or_else(|_| panic!("Не удалось создать/перезаписать файл: {}", &path))
			.write_all(&data)
			.expect("Не удалось записать данные в файл");

	} else {
		println!("Всего пикселей: {}", pixels.len());
		let max_bytes = pixels.len() * 6 / 8;
		println!("Доступно для записи: {}\n", get_z(max_bytes));

		let path = args.data.unwrap();
		println!("Чтение {}", &path);
		let data = fs::read(&path).unwrap_or_else(|_| panic!("Не удалось открыть {}", &path));
		println!("Размер файла: {}", get_z(data.len()));

		if data.len() > max_bytes {
			return println!("Файл превышает доступное место");
		}

		println!("Вшиваем файл в фото...");
		write_bits_to_pixels(&mut pixels, &data);

		println!("Обратное преобразование");
		let (width, _height) = image.dimensions();
		for (x, y, pixel) in image.enumerate_pixels_mut() {
			let index = (y * width + x) as usize;
			let (r, g, b) = pixels[index];
			*pixel = Rgb([r, g, b]);
		}

		let path = args.output.clone();
		image.save(&path).unwrap_or_else(|_| panic!("Не удалось сохранить изображение: {}", &path));
	}
}

fn main() {
	let result = panic::catch_unwind(|| {process()});
	match result {
		Ok(_) => println!("Программа завершилась без ошибок\n\n "),
		Err(e) => println!("{e:?}"),
	}
}
