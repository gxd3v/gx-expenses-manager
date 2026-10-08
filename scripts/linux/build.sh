set -eu

rm -rf /build
git clone --quiet /repo /build
cd /build
npm ci --no-audit --no-fund
npx tauri build --bundles deb,appimage --config src-tauri/tauri.release.conf.json

rm -rf /out/*
cp /target/release/bundle/appimage/*.AppImage* /target/release/bundle/deb/*.deb* /out/
