# Card game

## Setup
```bash
cargo install wasm-pack
cargo install wasm-bindgen-cli --version 0.2.89
sudo apt install inotify-tools
```


## Cross compiling to Windows
### Setup
```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
```
### Compile & package assets
```bash
./package_windows.sh
```