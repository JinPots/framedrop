# 📸 FrameDrop

<p align="center">
  <strong>Streamlined, professional RAW photo & video ingestion for high-end photography workflows.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Version-0.2.0-teal.svg" alt="Version">
  <img src="https://img.shields.io/badge/Framework-Tauri%202.0-blue.svg" alt="Tauri">
  <img src="https://img.shields.io/badge/Backend-Rust-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Frontend-Vanilla%20JS-38b2ac.svg" alt="Frontend">
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License">
</p>

---

[English](#english) | [Tiếng Việt](#tiếng-viet)

---

<div id="english"></div>

## 🚀 Overview

**FrameDrop** is a high-performance desktop utility designed for professional photographers who need a fast, reliable way to move media from camera storage to their workstations. Built with a "utility-first" philosophy, it automates the tedious parts of data transfer while ensuring your files are perfectly organized and verifiable.

## ✨ Key Features

### 🔍 Deep Metadata Extraction
- **Sony XAVC Support**: Automatically prioritizes professional XML sidecars (M01.XML) for camera model and date detection.
- **Organization**: Dynamically organizes media into `YYYY-MM-DD / [Camera Model] / [Files]` structures.

### 📊 Precision Transfer Engine
- **Byte-Accurate ETA**: Real-time time estimation based on actual data throughput (MB/s), not just file count.
- **Granular Progress**: Individual per-file progress bars alongside overall session tracking.
- **Safe Organization**: Intelligent folder suffixing (e.g., `ILCE-7M3 (1)`) to handle multiple ingests into the same destination without collisions.

### 🌐 Advanced Automation
- **Remote Sync**: Automatically kick off secondary backups or cloud syncs after local ingestion completes.
- **Webhooks**: Get rich Discord/Slack notifications with session summaries upon completion.
- **Auto-Updates**: Built-in update infrastructure to ensure you're always on the latest version.

### 🎨 Premium Interface
- **Minimalist Design**: A dark, pro-mark aesthetic centered around a custom-built physical camera logo.
- **Bi-lingual Support**: Full support for English and Tiếng Việt.

## 🛠️ Tech Stack

- **Engine**: [Rust](https://www.rust-lang.org/) for high-speed buffered file I/O and safe metadata parsing (via `lofty` and `kamadak-exif`).
- **Interface**: [Tauri 2.0](https://tauri.app/) for a hardware-accelerated, lightweight native shell.
- **Build System**: [Vite](https://vitejs.dev/) for sub-second hot module replacement.

---

<div id="tiếng-viet"></div>

## 🇻🇳 Tiếng Việt

## 🚀 Tổng quan

**FrameDrop** là một công cụ máy tính hiệu năng cao dành cho các nhiếp ảnh gia chuyên nghiệp, những người cần một cách nhanh chóng và đáng tin cậy để chuyển dữ liệu từ thẻ nhớ máy ảnh vào máy trạm. Được xây dựng với triết lý "tiện ích là trên hết", ứng dụng tự động hóa các phần tẻ nhạt của việc truyền dữ liệu trong khi đảm bảo các tệp của bạn được sắp xếp hoàn hảo.

## ✨ Các tính năng chính

### 🔍 Trích xuất siêu dữ liệu chuyên sâu
- **Hỗ trợ Sony XAVC**: Tự động ưu tiên các tệp XML sidecar (M01.XML) để nhận diện model máy ảnh và ngày chụp.
- **Sắp xếp**: Tự động phân loại phương tiện vào cấu trúc `YYYY-MM-DD / [Model Máy ảnh] / [Tệp]`.

### 📊 Công cụ truyền tín hiệu chính xác
- **Ước tính thời gian (ETA) theo Byte**: Ước tính thời gian thực dựa trên lưu lượng dữ liệu thực tế (MB/s), không chỉ dựa trên số lượng tệp.
- **Tiến trình chi tiết**: Thanh tiến trình riêng biệt cho từng tệp cùng với việc theo dõi toàn bộ phiên làm việc.
- **Sắp xếp an toàn**: Hậu tố thư mục thông minh (ví dụ: `ILCE-7M3 (1)`) để xử lý nhiều lần nhập vào cùng một đích mà không bị xung đột.

### 🌐 Tự động hóa nâng cao
- **Đồng bộ từ xa**: Tự động kích hoạt sao lưu phụ hoặc đồng bộ đám mây sau khi nhập dữ liệu cục bộ hoàn tất.
- **Webhooks**: Nhận thông báo phong phú trên Discord/Slack với bản tóm tắt phiên làm việc khi hoàn thành.
- **Tự động cập nhật**: Cơ sở hạ tầng cập nhật tích hợp để đảm bảo bạn luôn sử dụng phiên bản mới nhất.

## ⌨️ Bắt đầu (Development)

1. **Cài đặt các gói phụ thuộc**
   ```bash
   npm install
   ```

2. **Chạy môi trường phát triển**
   ```bash
   npm run tauri dev
   ```

3. **Xây dựng bản sản phẩm (Build)**
   ```bash
   npm run tauri build
   ```

---

## 🤝 Credits & Acknowledgments

- **Developer**: [JinPots](https://github.com/JinPots)
- **AI Specialist**: Built in collaboration with **Antigravity**, a powerful agentic AI coding assistant by Google DeepMind.

## 📄 License

Bản quyền thuộc về **MIT License** - xem tệp [LICENSE](LICENSE) để biết chi tiết.
