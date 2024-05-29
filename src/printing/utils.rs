use std::process::Command;

use serde::{Deserialize, Serialize};
use printers;



#[derive(Debug, Serialize, Deserialize)]
pub struct Printer {
    name: String,
    is_default: bool,
}

pub fn get_printers_list() -> Vec<Printer> {
  let mut parsed_printers= Vec::<Printer>::new();
  let printers = printers::get_printers();

  for printer in printers {
      let printer_item = Printer {
          name: printer.name,
          is_default: printer.is_default,
      };
      parsed_printers.push(printer_item);
  }

  return parsed_printers;
}

pub fn print_pdf(filepath: &str, printer_name: &str) -> std::process::Output {
  let output = if cfg!(target_os = "windows") {
      Command::new("PDFtoPrinter.exe")
          .args([filepath, printer_name])
          .output()
          .expect("failed to execute process")
  } else {
    // use lpr for mac and linux
      Command::new("lpr")
          .args(["-P", printer_name, filepath])
          .output()
          .expect("failed to execute process")
  };

  return output;
}
