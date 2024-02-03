set -e

mkdir -p windows_build
rm -rf windows_build/*

cargo build --target x86_64-pc-windows-gnu --release --no-default-features
ln -s ../assets windows_build/assets
cp target/x86_64-pc-windows-gnu/release/client.exe windows_build/
cp launch_game.bat windows_build/
mkdir windows_build/logs

cd windows_build
zipfile=windows_build_$(date +"%Y-%m-%d_%H-%M-%S").zip
zip -r "$zipfile" ./*

echo "./windows_build/${zipfile}"
