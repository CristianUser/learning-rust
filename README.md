# Printing Service

A lightweight HTTP printing service built in Rust that exposes a local REST API for submitting print jobs and querying available printers. It is designed to run as a background service (Windows Service or Linux systemd unit) on point-of-sale and back-office machines, allowing web or desktop applications to print without direct OS printer access.

**Maintainer:** Pronesoft SRL

---

## Features

- List all installed printers and their current state
- Print from multiple input formats:
  - **PDF** – base64-encoded PDF document
  - **HTML** – raw HTML string rendered to PDF via headless Chrome
  - **URL** – web page rendered to PDF via headless Chrome (supports Bearer token auth)
  - **RAW** – base64-encoded ESC/POS or other raw printer data (for thermal/POS printers)
- Runs as a **Windows Service** (`PronesoftPrintingService`) or **Linux systemd unit**
- CORS-permissive by default for easy local integration

---

## API

Base URL: `http://127.0.0.1:1830` (standalone) / `http://127.0.0.1:1829` (Windows service)

| Method | Path        | Description                        |
|--------|-------------|------------------------------------|
| GET    | `/health`   | Health check                       |
| GET    | `/printers` | List available printers            |
| POST   | `/print`    | Submit a print job (JSON body)     |

### POST `/print` body

```json
{
  "printer_name": "HP LaserJet Pro",
  "format": "pdf",
  "content": "<base64-encoded content>",
  "auth_token": ""
}
```

`format` accepts: `pdf`, `html`, `url`, `raw`.  
`auth_token` is only used when `format` is `url`.

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2021)
- For `html` / `url` printing: Chrome or Chromium must be installed
- **Windows only:** `PDFtoPrinter.exe` (included in `packaging/windows/`)
- **Linux only:** `lpr` / `lp` command-line tools

### Run in development mode

```bash
cargo run
```

The server starts on `http://127.0.0.1:1830`.

### Build release binaries

```bash
# Main service binary
cargo build --release

# Windows service wrapper
cargo build --release --bin win
```

---

## Packaging & Distribution

### Linux (Debian / Ubuntu)

Use the provided packaging script:

```bash
bash packaging/package.sh
```

This installs `cargo-deb` if needed, builds release binaries, and produces a `.deb` package at `target/debian/printing-service_*.deb`.

Install the package:

```bash
sudo dpkg -i target/debian/printing-service_*.deb
```

The systemd service is enabled and started automatically on install.

Manage the service manually:

```bash
sudo systemctl start printing-service
sudo systemctl stop printing-service
sudo systemctl status printing-service
```

### Windows Installer

1. Build release binaries on a Windows machine:

   ```powershell
   cargo build --release
   cargo build --release --bin win
   ```

2. Copy `PDFtoPrinter.exe` from `packaging/windows/` into `target/release/`.

3. Install [Inno Setup](https://jrsoftware.org/isinfo.php) and run:

   ```powershell
   iscc packaging\windows\inno-file.iss
   ```

   The installer is generated at `packaging/windows/PrintingServiceInstaller.exe`.

4. Running the installer registers `PronesoftPrintingService` as a Windows service that starts automatically on boot and listens on port `1829`.

Manage the service manually:

```powershell
net start PronesoftPrintingService
net stop PronesoftPrintingService
```

---

## License

See [packaging/LICENSE](packaging/LICENSE).
