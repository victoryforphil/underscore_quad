#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
bin_name="underscore_quad"
install_root="${HOME}/.local/opt/${bin_name}"
bin_path="${install_root}/${bin_name}"
desktop_dir="${HOME}/.local/share/applications"
desktop_file="${desktop_dir}/${bin_name}.desktop"

mkdir -p "$install_root" "$desktop_dir"
install -m 755 "${script_dir}/${bin_name}" "$bin_path"

cat > "$desktop_file" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=underscore_quad
Comment=Low-latency UVC camera viewer
Exec=${bin_path}
Terminal=false
Categories=AudioVideo;Utility;
StartupNotify=true
EOF

chmod 644 "$desktop_file"

printf 'Installed %s to %s\n' "$bin_name" "$bin_path"
printf 'Desktop entry written to %s\n' "$desktop_file"
printf 'On Steam Deck, add the desktop entry to Steam in Desktop Mode if you want Game Mode launching.\n'
