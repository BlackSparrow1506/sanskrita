#!/bin/bash
# संस्कृता installer — creates the `sanskrita` command.
# Usage:  bash install.sh
set -e

HERE="$(cd "$(dirname "$0")" && pwd)"

if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 न प्राप्तम् / python3 not found — please install Python 3 first."
    exit 1
fi

# pick a bin directory
if [ -w /usr/local/bin ]; then
    BIN=/usr/local/bin
else
    BIN="$HOME/.local/bin"
    mkdir -p "$BIN"
fi

cat > "$BIN/sanskrita" <<EOF
#!/bin/bash
exec python3 "$HERE/sanskrita.py" "\$@"
EOF
chmod +x "$BIN/sanskrita"

cat > "$BIN/sanskrita-playground" <<EOF
#!/bin/bash
exec python3 "$HERE/playground.py" "\$@"
EOF
chmod +x "$BIN/sanskrita-playground"

echo "स्थापितम् ✓  Installed:"
echo "  sanskrita <file.सं>      — run a program"
echo "  sanskrita               — REPL"
echo "  sanskrita-playground    — browser playground"
if [ "$BIN" = "$HOME/.local/bin" ] && ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    echo ""
    echo "NOTE: add this to your ~/.zshrc so the command is found:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
echo ""
echo "प्रथमः कार्यक्रमः / first program:"
echo "  sanskrita \"$HERE/examples/नमस्ते.सं\""
