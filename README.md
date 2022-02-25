# Open2Internet2

The second rust rewrite of the original O2I application written in C.

It uses GTK4 frontend.

## Installation

### Windows
- Download from the [Releases](https://github.com/MGlolenstine/open2internet2/releases)
- Extract and run the `open2internet.exe`

## Installation from source

### Linux
- Install Rust programming language
- Install the binary using `cargo`
```
cargo install --git https://github.com/MGlolenstine/open2internet2.git
```

### Windows
- Install docker
- Clone the repository
```
git clone https://github.com/MGlolenstine/open2internet2.git
```
- Open the directory
- Start the Docker image [gtk4-cross](https://github.com/MGlolenstine/gtk4-cross) using the following command
```
docker -ti -v <PATH>:/mnt ghcr.io/mglolenstine/gtk4-cross:rust-gtk-4.6
```
- Type `build` and wait for the build to finish
- Type `package_with_icons` and wait for the packaging to finish.
- Your executable with all dynamic libraries can be found in the root of the project archived in `package.zip`.
