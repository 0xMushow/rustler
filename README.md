# Rustler

**Rustler** is a backend service for file uploads, processing, and storage, built with **Rust**. It uses **AWS S3** for file storage, **Redis** for task queuing, and **Amazon RDS** for metadata storage. The project is designed to handle large files efficiently, process them asynchronously, and store relevant metadata in a scalable database.

### Why "Rustler"?

The name "Rustler" is a fun play on words. It’s a contraction of **Rust Hustler**, meaning that just like a hustler gets things done quickly and efficiently, **Rustler** does the same with your file uploads and background processing—all powered by the speed and power of **Rust**!

### Features

- **File Upload**: Upload files and store them in **AWS S3**.
- **Asynchronous Processing**: Process files using **Tokio** for async task handling.
- **Metadata Storage**: Store file metadata (name, size, timestamp) in **Amazon RDS** (PostgreSQL) for easy scalability and management.
- **Task Queue**: Manage background tasks with **Redis**, ensuring scalable, non-blocking file processing.
- **Real-Time Updates**: Receive updates on the processing status of your files through simple API endpoints.

### Installation

To get started with **Rustler**, follow these steps:

1. **Clone the repository**:
   ```bash
   git clone https://github.com/0xMushow/rustler.git
   ```

2. **Navigate to the project directory**:
   ```bash
   cd rustler
   ```

3. **Run the project**:
   ```bash
   cargo run
   ```

4. **Configure AWS S3, RDS, and Redis**:
    - Make sure to configure your **AWS S3** bucket, **Amazon RDS** (for PostgreSQL), and **Redis** server credentials before running the application.

5. **Docker (Optional)**:
    - Dockerize the entire application for easy local development or deployment:
      ```bash
      docker-compose up
      ```

### License

This project is licensed under the **MIT License**. See the [LICENSE](LICENCE) file for more details.