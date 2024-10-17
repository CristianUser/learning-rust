use std::{collections::HashMap, io::Write, path};
use base64::{Engine as _, engine::general_purpose};
use headless_chrome::{self, types::PrintToPdfOptions};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

mod utils;

fn random_string(length: usize) -> String {
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    let rand_string: String = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(length)
    .map(char::from)
    .collect();

    return rand_string
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    message: String,
}

fn get_temporary_file_path() -> String {
    // random temp file name
    let filename = format!("output_{}.pdf", random_string(7));
    let mut path = std::env::temp_dir();
    path.push(filename);
    path.to_str().unwrap().to_owned()
}

#[get("/printers")]
pub async fn list_printers() -> impl Responder {
    HttpResponse::Ok().json(utils::get_printers_list())
}

#[derive(Deserialize)]
struct PrintJobInput {
    printer_name: String,
    content: String,
    format: String,
    auth_token: String, // only used if format is url
}

#[post("/print")]
pub async fn print(job: web::Json<PrintJobInput>) -> impl Responder {
    let printer_name = &job.printer_name;
    let filename = &get_temporary_file_path();
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

    if job.format == "url" {
        let browser = headless_chrome::Browser::default().unwrap();
        let tab = browser.new_tab().unwrap();
        if !job.auth_token.is_empty() {
            let auth_header = format!("Bearer {}", job.auth_token);
            let mut headers = HashMap::new();
            headers.insert("Authorization", auth_header.as_str());
            let result = tab.set_extra_http_headers(headers);
            if result.is_err() {
                return Err(actix_web::error::ErrorInternalServerError("Failed to set headers"));
            }
        }
        tab.navigate_to(&job.content).unwrap();
        tab.wait_until_navigated().unwrap();
        let content = tab.print_to_pdf({
            let options = PrintToPdfOptions {
                prefer_css_page_size: Some(true),
                ..Default::default()
            };
            Some(options)
        }).unwrap();
        let mut file = std::fs::File::create(filename).unwrap();
        file.write_all(content.as_slice()).unwrap();
    }

    if job.format == "pdf" {
        let mut file = std::fs::File::create(filename).unwrap();
        // convert base64 to bytes
        let content = general_purpose::STANDARD.decode(&job.content).unwrap();
        file.write_all(content.as_slice()).unwrap();
    }
    
    // write content to file for debugging
    let output = utils::print_pdf(filename, printer_name);
    if !output.status.success() {
        println!("{:?}", output.stderr);
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
