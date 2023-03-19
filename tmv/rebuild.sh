set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd $SCRIPT_DIR

wasm-pack build --target web --out-dir pkg
cp ./pkg/tmv_bg.wasm ../web/public/
