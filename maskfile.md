# Virus

> 🦠

<!-- ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ -->

## bacon

> Runs `bacon mask -- bacon fmtcheck`

You need a command called `mask` calling `mask` in your `bacon` config.

```sh
bacon mask -- bacon fmtcheck
```

### fmtcheck

> Formats and checks workspace

```sh
cargo fmt --quiet --all
cargo check --workspace --all-targets
```

<!-- ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ -->

## watch

> Reruns on exit

**OPTIONS**

- release
  - flags: --release
  - type: boolean
  - desc: Compiles in release mode

```sh
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

while true; do
    if cargo build; then
        if [ "$release" = "true" ]; then
            cargo run --release
        else
            cargo run
        fi
    else
        echo "${RED}Running old binary${NC}"
        ./target/debug/virus
    fi

    echo "${BLUE}Reruns in 1s (press any key to exit)${NC}"
    read -n 1 -t 1 key && exit
done
```
