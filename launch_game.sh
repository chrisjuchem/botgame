set -e

dev=0
config="./assets/config1.json"
exe="client"

while [ -n "$1" ] ; do
  case "$1" in
    "--dev")
      dev=1
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
  tput reset && RUST_BACKTRACE=1 cargo run --bin "$exe" "$config"
else
  mkdir -p logs
  # TODO test
  RUST_BACKTRACE=1 "./${exe}" "${config}" 2>&1 | tee "logs/$(date +%Y-%m-%d_%H-%M-%S).log"
fi
