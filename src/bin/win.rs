// Ping service example.
//
// You can install and uninstall this service using other example programs.
// All commands mentioned below shall be executed in Command Prompt with Administrator privileges.
//
// Service installation: `install_service.exe`
// Service uninstallation: `uninstall_service.exe`
//
// Start the service: `net start printing_service`
// Stop the service: `net stop printing_service`
//
// Ping server sends a text message to local UDP port 1234 once a second.
// You can verify that service works by running netcat, i.e: `ncat -ul 1234`.
#[cfg(windows)]
fn main() -> windows_service::Result<()> {
    printing_service::run()
}

#[cfg(not(windows))]
fn main() {
    panic!("This program is only intended to run on Windows.");
}

#[cfg(windows)]
mod printing_service {
    use parking_lot::Mutex;
    use std::{ffi::OsString, sync::mpsc::{self, Receiver}, thread, time::Duration};
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher, Result,
    };

    const SERVICE_NAME: &str = "PronesoftPrintingService";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    use actix_cors::Cors;
    use actix_web::{
        dev::ServerHandle, get, middleware, web, App, HttpResponse, HttpServer, Responder
    };

    #[path = "../../printing/routes.rs"]
    mod routes;
    use routes::{list_printers, print};
    use serde::Serialize;

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    #[derive(Debug, Serialize)]
    struct Health {
        status: String,
        version: String,
    }

    #[get("/health")]
    async fn health() -> impl Responder {
        HttpResponse::Ok().json(Health {
            status: "ok".to_owned(),
            version: VERSION.to_owned(),
        })
    }

    #[actix_web::main]
    pub async fn start_server(shutdown_rx: Receiver<()>) -> std::io::Result<()> {
        // create the stop handle container
        let stop_handle = web::Data::new(StopHandle::default());

        let srv = HttpServer::new({
            let stop_handle = stop_handle.clone();

            move || {
                let cors = Cors::permissive(); // Change this to configure your CORS settings
                                               // give the server a Sender in .data
                App::new()
                    .wrap(cors)
                    .app_data(stop_handle.clone())
                    .service(health)
                    .service(list_printers)
                    .service(print)
                    .wrap(middleware::Logger::default())
            }
        })
        .bind(("127.0.0.1", 1829))?
        .run();
        println!("Listening on port 1829");

        // register the server handle with the stop handle
        stop_handle.register(srv.handle());

        // Poll shutdown event in a separate thread to avoid blocking the main thread and receive requests
        thread::spawn(move || {
            while let Ok(_) = shutdown_rx.recv() {
                stop_handle.stop(true);
                // Break the loop either upon stop or channel disconnect
                break;
            }
        });

        srv.await
    }

    #[derive(Default)]
    struct StopHandle {
        inner: Mutex<Option<ServerHandle>>,
    }

    impl StopHandle {
        /// Sets the server handle to stop.
        pub(crate) fn register(&self, handle: ServerHandle) {
            *self.inner.lock() = Some(handle);
        }

        /// Sends stop signal through contained server handle.
        pub(crate) fn stop(&self, graceful: bool) {
            #[allow(clippy::let_underscore_future)]
            let _ = self.inner.lock().as_ref().unwrap().stop(graceful);
        }
    }

    pub fn run() -> Result<()> {
        // Register generated `ffi_service_main` with the system and start the service, blocking
        // this thread until the service is stopped.
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    // Generate the windows service boilerplate.
    // The boilerplate contains the low-level service entry function (ffi_service_main) that parses
    // incoming service arguments into Vec<OsString> and passes them to user defined service
    // entry (my_service_main).
    define_windows_service!(ffi_service_main, my_service_main);

    // Service entry function which is called on background thread by the system with service
    // parameters. There is no stdout or stderr at this point so make sure to configure the log
    // output to file if needed.
    pub fn my_service_main(_arguments: Vec<OsString>) {
        if let Err(_e) = run_service() {
            // Handle the error, by logging or something.
        }
    }

    pub fn run_service() -> Result<()> {
        // Create a channel to be able to poll a stop event from the service worker loop.
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        // Define system service event handler that will be receiving service events.
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Notifies a service to report its current status information to the service
                // control manager. Always return NoError even if not implemented.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                // Handle stop
                ServiceControl::Stop => {
                    shutdown_tx.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }

                // treat the UserEvent as a stop request
                ServiceControl::UserEvent(code) => {
                    if code.to_raw() == 130 {
                        shutdown_tx.send(()).unwrap();
                    }
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        // Register system service event handler.
        // The returned status handle should be used to report service status changes to the system.
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Tell the system that service is running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        // Start the service worker loop.
        let _ = start_server(shutdown_rx);

        // Tell the system that service has stopped.
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }
}
