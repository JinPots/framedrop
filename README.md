# 📸 FrameDrop

<p align="center">
  <strong>Streamlined, automated RAW photo ingestion for professional photographers.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Version-0.1.0-teal.svg" alt="Version">
  <img src="https://img.shields.io/badge/Framework-Tauri%202.0-blue.svg" alt="Tauri">
  <img src="https://img.shields.io/badge/Backend-Rust-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/Frontend-Vanilla%20JS%20%2B%20Tailwind-38b2ac.svg" alt="Frontend">
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License">
</p>

---

[English](#english) | [Tiếng Việt](#tiếng-việt)

---

<div id="english"></div>

## 🚀 Overview

**FrameDrop** is a high-performance desktop application designed to bridge the gap between your camera's storage and your editing workstation. By monitoring your system for removable media, FrameDrop automatically identifies RAW image files and organizes them into a structured directory hierarchy based on EXIF metadata (Date & Camera Model).

## ✨ Key Features

- **⚡ Instant Ingestion**: Automatically detects SD cards and camera storage upon connection.
- **📂 Smart Organization**: Dynamically maps files to folders like `YYYY-MM-DD / [Camera Model] / [Files]`.
- **📊 Real-time Progress**: A sleek, reactive dashboard to track active transfers and history.
- **🎨 Premium UI**: A modern, dark-mode native interface built for speed and visual clarity.
- **⚙️ Configurable**: Easily define destination paths and organization rules.

## 🛠️ Tech Stack

- **Engine**: [Rust](https://www.rust-lang.org/) for multi-threaded performance and safety.
- **Interface**: [Tauri 2.0](https://tauri.app/) for a lightweight, native cross-platform experience.
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) for a refined, responsive design.
- **Build System**: [Vite](https://vitejs.dev/) for high-speed frontend development.

## ⌨️ Getting Started

### Prerequisites

- **Node.js** (LTS recommended)
- **Rust Toolchain** (via `rustup`)
- System dependencies for Tauri (Refer to the [Tauri Setup Guide](https://tauri.app/v1/guides/getting-started/prerequisites))

### Installation & Development

1. **Clone & Install Dependencies**
   ```bash
   npm install
   ```

2. **Launch Dev Environment**
   ```bash
   npm run tauri dev
   ```

3. **Build Production Executable**
   ```bash
   npm run tauri build
   ```

---

<div id="tiếng-việt"></div>

## 🇻🇳 Tiếng Việt

## 🚀 Tổng quan

**FrameDrop** là một ứng dụng máy tính hiệu năng cao được thiết kế để kết nối bộ nhớ máy ảnh với máy trạm chỉnh sửa của bạn. Bằng cách giám sát hệ thống để tìm các thiết bị lưu trữ rời, FrameDrop tự động nhận dạng các tệp ảnh RAW và sắp xếp chúng vào cấu trúc thư mục dựa trên siêu dữ liệu EXIF (Ngày & Model Máy ảnh).

## ✨ Các tính năng chính

- **⚡ Nhập liệu tức thì**: Tự động phát hiện thẻ SD và bộ nhớ máy ảnh khi kết nối.
- **📂 Sắp xếp thông minh**: Tự động phân loại tệp vào các thư mục theo định dạng `YYYY-MM-DD / [Model Máy ảnh] / [Tệp]`.
- **📊 Tiến trình thời gian thực**: Bảng điều khiển hiện đại, phản hồi nhanh để theo dõi quá trình sao chép và lịch sử.
- **🎨 Giao diện cao cấp**: Giao diện gốc chế độ tối (dark-mode) hiện đại, được xây dựng cho tốc độ và sự rõ ràng về thị giác.
- **⚙️ Có thể cấu hình**: Dễ dàng xác định đường dẫn đích và các quy tắc sắp xếp.

## 🛠️ Công nghệ sử dụng

- **Engine**: [Rust](https://www.rust-lang.org/) cho hiệu năng đa luồng và sự an toàn.
- **Giao diện**: [Tauri 2.0](https://tauri.app/) cho trải nghiệm ứng dụng gốc nhẹ nhàng, đa nền tảng.
- **Thiết kế**: [Tailwind CSS](https://tailwindcss.com/) cho thiết kế tinh tế và phản hồi nhanh.
- **Build System**: [Vite](https://vitejs.dev/) để phát triển frontend tốc độ cao.

## ⌨️ Bắt đầu

### Điều kiện tiên quyết

- **Node.js** (Khuyến nghị bản LTS)
- **Rust Toolchain** (thông qua `rustup`)
- Các phụ thuộc hệ thống cho Tauri (Tham khảo [Hướng dẫn thiết lập Tauri](https://tauri.app/v1/guides/getting-started/prerequisites))

### Cài đặt & Phát triển

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

This project was developed with a focus on combining performance-grade backend logic with modern design aesthetics.

- **Developer**: [JinPots](https://github.com/JinPots)
- **AI Specialist**: Built in collaboration with **Antigravity**, a powerful agentic AI coding assistant by Google DeepMind.

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.
