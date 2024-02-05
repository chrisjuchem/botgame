set -e

dev=0
config=""

while [ -n "$1" ] ; do
  case "$1" in
    "--dev")
      dev=1
      ;;
    *)
      config="$1"
  esac
  shift
done

if [ $dev -eq 1 ]; then
  config="assets/config${config}.json"
  tput reset && RUST_BACKTRACE=1 cargo run --bin client "$config"
else
  mkdir -p logs
  # TODO test
  RUST_BACKTRACE=1 ./client "${config:-assets/config1.json}" 2>&1 | tee "logs/$(date +%Y-%m-%d_%H-%M-%S).log"
fi
