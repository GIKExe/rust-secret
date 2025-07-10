use clap::Parser;
use std::{fs, io::{Cursor, Read, Write}, panic};

mod png;

#[derive(Parser)]
#[command(
	name = "secret",
	version = "1.0",
	about = "Прячет данные в PNG RGB (24 бит)",
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
		format!("{num} Б")
	}
}

fn write_bits_to_bytes(bytes: &mut Vec<u8>, data: &[u8]) {
	let mut data = data.iter();
	let mut sh = 0u8;
	let mut byte = *data.next().unwrap_or(&0);
	for color in bytes {
		*color = (*color & 252) | ((byte >> sh) & 3);
		sh += 2;
		if sh > 7 {
			byte = *data.next().unwrap_or(&0);
			sh = 0
		};
	}
}

fn read_bits_from_bytes(bytes: &Vec<u8>) -> Vec<u8> {
	let mut data = Vec::new();
	let mut sh = 0u8;
	let mut byte = 0;
	for color in bytes {
		byte |= (*color & 3) << sh;
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

	let path = args.input.clone();
	println!("Чтение {}", &path);
	let (mut bytes, info) = png::read_image(&path);

	if args.data.is_none() {
		println!("Читаем файл из фото...");
		let mut buf = Cursor::new(read_bits_from_bytes(&bytes));
		let mut size = [0u8, 0, 0, 0];
		buf.read_exact(&mut size).expect("Ошибка декодирования файла");
		let size = u32::from_le_bytes(size) as usize;
		let mut data = vec![0u8; size];
		buf.read_exact(&mut data).expect("Ошибка декодирования файла");

		let path = args.output.clone();
		fs::File::create(&path)
			.unwrap_or_else(|_| panic!("Не удалось создать/перезаписать файл: {}", &path))
			.write_all(&data)
			.expect("Не удалось записать данные в файл");

	} else {

		println!("Всего пикселей: {}", bytes.len());
		let max_bytes = bytes.len() * 2 / 8;
		println!("Доступно для записи: {}\n", get_z(max_bytes));

		let path = args.data.unwrap();
		println!("Чтение {}", &path);
		let mut filedata = fs::read(&path).unwrap_or_else(|_| panic!("Не удалось открыть {}", &path));
		let mut data = (filedata.len() as u32).to_le_bytes().to_vec();
		println!("Размер файла: {}", get_z(filedata.len()));
		data.append(&mut filedata);

		if data.len() > max_bytes {
			return println!("Файл превышает доступное место");
		}

		println!("Вшиваем файл в фото...");
		write_bits_to_bytes(&mut bytes, &data);

		println!("Сохранение файла...");
		let path = args.output.clone();
		png::write_image(path, &bytes, info);
	}
}

fn main() {
	let result = panic::catch_unwind(process);
	match result {
		Ok(_) => println!("Программа завершилась без ошибок\n\n "),
		Err(e) => println!("{e:?}"),
	}
}
