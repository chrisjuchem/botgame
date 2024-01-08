
trap 'kill $pyserver' EXIT

python3 -m http.server 8001 &
pyserver=$!

xdg-open http://0.0.0.0:8001

while true; do
  # rebuild
  wasm-pack build --no-pack --no-typescript --target web --dev

  # wait for a change to source files
  inotifywait --exclude pkg/* --exclude target/* -e modify -e create -e delete ./*
done
