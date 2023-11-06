SOURCE_IMAGE="icon.png"
mkdir -p AppIcon.iconset
sips -z 16 16     "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_16x16.png
sips -z 32 32     "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_16x16@2x.png
sips -z 32 32     "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_32x32.png
sips -z 64 64     "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_32x32@2x.png
sips -z 128 128   "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_128x128.png
sips -z 256 256   "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_128x128@2x.png
sips -z 256 256   "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_256x256.png
sips -z 512 512   "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_256x256@2x.png
sips -z 512 512   "${SOURCE_IMAGE}" --out AppIcon.iconset/icon_512x512.png
cp "${SOURCE_IMAGE}" AppIcon.iconset/icon_512x512@2x.png
iconutil -c icns AppIcon.iconset
## move it into the app bundle
mv AppIcon.icns ./RustyMandelbrot.app/Contents/Resources
