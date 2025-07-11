use clap::Parser;
use std::{fs, io::{Cursor, Read}};

mod error;
use error::Error;
mod io;

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

fn process() -> Result<(), Error>{
	let args = Args::parse();

	let path = args.input.clone();
	println!("Чтение {}", &path);
	let (mut bytes, info) = io::read_image(&path)?;

	if let Some(path) = args.data {
		println!("Всего пикселей: {}", bytes.len());
		let max_bytes = bytes.len() * 2 / 8;
		println!("Доступно для записи: {}\n", get_z(max_bytes));

		// let path = args.data.unwrap();
		println!("Чтение {}", &path);
		let mut filedata = fs::read(&path).map_err(|_| Error::FileRead(path.clone()))?;
		let mut data = (filedata.len() as u32).to_le_bytes().to_vec();
		println!("Размер файла: {}", get_z(filedata.len()));
		data.append(&mut filedata);

		println!("Вшиваем файл в фото...");
		if data.len() > max_bytes {return Err(Error::NoFreeSpace)};
		write_bits_to_bytes(&mut bytes, &data);

		println!("Сохранение файла...");
		let path = args.output.clone();
		io::write_image(path, &bytes, info)?;

	} else {

		println!("Читаем файл из фото...");
		let mut buf = Cursor::new(read_bits_from_bytes(&bytes));
		let mut size = [0u8, 0, 0, 0];
		buf.read_exact(&mut size).map_err(|_| Error::BufEndedUnexpectedly)?;
		let size = u32::from_le_bytes(size) as usize;
		let mut data = vec![0u8; size];
		buf.read_exact(&mut data).map_err(|_| Error::BufEndedUnexpectedly)?;

		let path = args.output.clone();
		fs::write(&path, data).map_err(|_| Error::FileWrite(path.clone()))?;
	};
	Ok(())
}

fn main() {
	match process() {Ok(_) => {}, Err(e) => println!("{e}")};
}
