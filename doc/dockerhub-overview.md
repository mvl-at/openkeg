# OpenKeg Multi-Arch Docker Image

## Description
The OpenKeg Multi-Arch Docker Image is designed to build and run the 'openkeg' application on multiple architectures. This Docker image leverages the power of Rust and Alpine Linux to provide a lightweight and efficient environment for hosting OpenKeg. The image supports both ARM64 and AMD64 architectures, making it versatile and suitable for various deployment scenarios.

## Features
- Supports ARM64 and AMD64 architectures for broader compatibility.
- Based on Rust and Alpine Linux, ensuring a lightweight and efficient container environment.
- Built-in multi-stage build process for optimal image size and security.
- Customizable log levels and tracing for monitoring application behavior.
- Automatically exposes port 1926 for incoming connections.

## Usage
To use the OpenKeg Multi-Arch Docker Image, follow these steps:

1. Pull the Docker image:
   ```sh
   docker pull mvlat/openkeg:latest
   ```
1. Run the container with the desired configuration:
   ```sh
   docker run -d -p 1926:1926 -v /path/to/data:/data mvlat/openkeg
   ```
The `-p` flag maps port 1926 inside the container to the host, enabling incoming connections.
The `-v` flag allows you to mount a host directory as a volume for persistent data storage.

## Environment Variables:
The OpenKeg Docker Image supports the following environment variables:
- `RUST_BACKTRACE`: Set to 1 to enable backtraces in case of errors.
- `RUST_LOG`: Specifies the log level (e.g., debug, info, error).
- `RUST_LOG_STYLE`: Specifies the log style (e.g., always, auto, never).

## Build Configuration:
The build process is designed to be versatile, allowing you to specify the target architecture during the build. To build the image for a specific architecture, use the following command:

```sh
docker buildx build -t mvlat/openkeg:latest --platform=linux/amd64 --platform=linux/arm64 --progress=plain . --push
```

## Authors:

- Richard Stöckl (GitHub: @Eiskasten)

Note:
This Docker image is continuously maintained and updated by the community to ensure compatibility with the latest versions of Rust and Alpine Linux. Contributions and feedback are welcome via GitHub. Happy OpenKegging! 🍻