#!/usr/bin/env bash
set -euo pipefail

version="${1:?version is required}"
version="${version#v}"
root="$(cd "$(dirname "$0")/.." && pwd)"
native="$root/native"
mpv_lib="$native/fluxa-mpv/lib"
dist="$root/dist"
stage="$root/dist/stage"

case "$(uname -s)" in
    Linux*) os=linux ;;
    Darwin*) os=macos ;;
    MINGW*|MSYS*|CYGWIN*) os=windows ;;
    *) echo "unsupported host $(uname -s)" >&2; exit 1 ;;
esac
arch="$(uname -m | sed 's/aarch64/arm64/; s/amd64/x86_64/')"
name="fluxa-$version-$os-$arch"

rm -rf "$dist"
mkdir -p "$stage"

cargo build --release --manifest-path "$native/Cargo.toml" -p fluxa-desktop
bin="$native/target/release/fluxa-desktop"
[[ "$os" == windows ]] && bin="$bin.exe"

bundled_libs() {
    find "$mpv_lib" -maxdepth 1 \( -type f -o -type l \) \
        ! -name '*backup*' ! -name '*.new' ! -name '*.orig' ! -name 'pre-*' "$@"
}

package_linux() {
    local app="$stage/fluxa"
    mkdir -p "$app/lib"
    install -m 755 "$bin" "$app/fluxa-desktop"
    bundled_libs -exec cp -a {} "$app/lib/" \;
    tar -C "$stage" -czf "$dist/$name.tar.gz" fluxa
    if [[ "$arch" == x86_64 ]]; then
        "$root/scripts/package-linux-installers.sh" "$app" "$version" "$dist"
    fi
}

package_windows() {
    local app="$stage/Fluxa"
    mkdir -p "$app"
    cp "$bin" "$app/fluxa-desktop.exe"
    cp "$mpv_lib"/*.dll "$app/"
    (cd "$stage" && 7z a -tzip "$dist/$name.zip" Fluxa >/dev/null)
    local win_app win_dist
    win_app="$(cygpath -w "$app")"
    win_dist="$(cygpath -w "$dist")"
    ISCC="$(command -v iscc || echo "/c/Program Files (x86)/Inno Setup 6/ISCC.exe")"
    MSYS2_ARG_CONV_EXCL="*" "$ISCC" "/DAppVersion=$version" "/DSourceDir=$win_app" "/DOutputDir=$win_dist" "$(cygpath -w "$root/packaging/windows/fluxa.iss")"
    wix build "$(cygpath -w "$root/packaging/windows/fluxa.wxs")" -arch x64 \
        -d "Version=${version%%-*}" -d "SourceDir=$win_app" -o "$win_dist\\$name.msi"
}

package_macos() {
    local app="$stage/Fluxa.app"
    local contents="$app/Contents"
    mkdir -p "$contents/MacOS" "$contents/Resources/lib"
    install -m 755 "$bin" "$contents/MacOS/fluxa-desktop"
    cp -a "$mpv_lib"/*.dylib "$contents/Resources/lib/"
    make_icns "$contents/Resources/Fluxa.icns"
    cat > "$contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>Fluxa</string>
    <key>CFBundleDisplayName</key><string>Fluxa</string>
    <key>CFBundleIdentifier</key><string>com.fluxa.desktop</string>
    <key>CFBundleExecutable</key><string>fluxa-desktop</string>
    <key>CFBundleIconFile</key><string>Fluxa</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>$version</string>
    <key>CFBundleVersion</key><string>$version</string>
    <key>LSMinimumSystemVersion</key><string>12.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>LSApplicationCategoryType</key><string>public.app-category.entertainment</string>
</dict>
</plist>
PLIST
    codesign --force --deep --sign - "$app"
    ditto -c -k --keepParent "$app" "$dist/$name.zip"
    ln -s /Applications "$stage/Applications"
    hdiutil create -volname Fluxa -srcfolder "$stage" -ov -format UDZO "$dist/$name.dmg" >/dev/null
}

make_icns() {
    local out="$1" set="$stage/Fluxa.iconset" size
    mkdir -p "$set"
    for size in 16 32 64 128 256 512 1024; do
        rsvg-convert -w "$size" -h "$size" "$root/shared/brand/fluxa-mark.svg" -o "$set/raw_$size.png"
    done
    for size in 16 32 128 256 512; do
        cp "$set/raw_$size.png" "$set/icon_${size}x${size}.png"
        cp "$set/raw_$((size * 2)).png" "$set/icon_${size}x${size}@2x.png"
    done
    rm -f "$set"/raw_*.png
    iconutil -c icns "$set" -o "$out"
    rm -rf "$set"
}

"package_$os"
rm -rf "$stage"
(cd "$dist" && ls -1)
