
use std::fmt;
use colored::Colorize;

pub enum Error {
	FileRead(String),
	FileWrite(String),
	Decoder(String),
	Encoder(String),
	NoFreeSpace,
	BufEndedUnexpectedly,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::FileRead(e) => write!(f, "{}", format!("Не удалось прочистать файл: {e}").red()),
			Self::FileWrite(e) => write!(f, "{}", format!("Не удалось записать файл: {e}").red()),
			Self::Decoder(e) => write!(f, "{}", format!("Ошибка декодера: {e}").red()),
			Self::Encoder(e) => write!(f, "{}", format!("Ошибка кодировщика: {e}").red()),
			Self::NoFreeSpace => write!(f, "{}", "Недостаточно места для вшивания файла".red()),
			Self::BufEndedUnexpectedly => write!(f, "{}", "Буффер неожиданно закончился".red()),
		}
	}
}