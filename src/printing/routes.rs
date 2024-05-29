use std::{io::Write, path};
use headless_chrome;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

mod utils;

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    message: String,
}

#[get("/printers")]
pub async fn list_printers() -> impl Responder {
    HttpResponse::Ok().json(utils::get_printers_list())
}

#[derive(Deserialize)]
struct PrintJobInput {
    printer_name: String,
    content: String,
    format: String
}

#[post("/print")]
pub async fn print(job: web::Json<PrintJobInput>) -> impl Responder {
    let printer_name = &job.printer_name;
    let filename = "./output.pdf";
    // let content = &job.content;

    let printer = printers::get_printer_by_name(printer_name);
    if printer.is_none() {
        return Err(actix_web::error::ErrorNotFound("Printer not found"));
    }

    if job.format == "html" {
        let browser = headless_chrome::Browser::default().unwrap();
        let tab = browser.new_tab().unwrap();
        let data = format!("data:text/html,{}", &job.content.to_owned());
        tab.navigate_to(&data).unwrap();
        tab.wait_until_navigated().unwrap();
        let content = tab.print_to_pdf(None).unwrap();
        let mut file = std::fs::File::create(filename).unwrap();
        file.write_all(content.as_slice()).unwrap();

    }
    
    // write content to file for debugging
    let output = utils::print_pdf(filename, printer_name);
    if !output.status.success() {
        println!("{:?}", output);
        return Err(actix_web::error::ErrorInternalServerError("Failed to print"));
    }

    // remove file after printing
    let path = path::Path::new(filename);
    if path.exists() {
        std::
        fs::remove_file(path).unwrap();
    }

    Ok(HttpResponse::Ok().json(Message {
        message: "Printing".to_owned(),
    }))
}
