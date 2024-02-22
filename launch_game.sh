set -e

dev=0
config="./assets/config1.json"
exe="client"
cargo_flags=""

while [ -n "$1" ] ; do
  case "$1" in
    "--dev")
      dev=1
      ;;
    "--trace")
      cargo_flags="${cargo_flags} --features trace"
      ;;
    "--server")
      exe="server"
      config=""
      ;;
    *)
      config="$1"
  esac
  shift
done

if [ $dev -eq 1 ]; then
  if [ -n "$config" ] ; then
    config="assets/config${config}.json"
  fi
  # intentionally split cargo flags
  tput reset && RUST_BACKTRACE=1 cargo run $cargo_flags --bin "$exe" "$config"
else
  mkdir -p logs
  # TODO test
  RUST_BACKTRACE=1 "./${exe}" "${config}" 2>&1 | tee "logs/$(date +%Y-%m-%d_%H-%M-%S).log"
fi
