use std::{fmt::Debug, io::Write, path, process::Command};

use printers;
use headless_chrome;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Printer {
    name: String,
    is_default: bool,
}

fn print_pdf(filepath: &str, printer_name: &str) -> std::process::Output {
    let output = if cfg!(target_os = "windows") {
        Command::new("./PDFtoPrinter.exe")
            .args([filepath, printer_name])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("echo hello")
            .output()
            .expect("failed to execute process")
    };

    return output;
}

fn get_printers_list() -> Vec<Printer> {
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


#[derive(Debug, Serialize, Deserialize)]
struct Message {
    message: String,
}


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json(Message {
        message: "Hello, World!".to_owned(),
    })
}

#[get("/printers")]
async fn list_printers() -> impl Responder {
    HttpResponse::Ok().json(get_printers_list())
}

#[derive(Deserialize)]
struct PrintJobInput {
    printer_name: String,
    content: String,
    format: String
}

#[post("/print")]
async fn print(job: web::Json<PrintJobInput>) -> impl Responder {
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
    let output = print_pdf(filename, printer_name);
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

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(list_printers)
            .service(print)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
