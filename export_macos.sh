# set the name of the Mac App
APP_NAME="BevyAudioviz"
# set the name of your rust crate
RUST_CRATE_NAME="bevy_audioviz"
# create the folder structure
mkdir -p "exports/macos/${APP_NAME}.app/Contents/MacOS"
mkdir -p "exports/macos/${APP_NAME}.app/Contents/Resources"
# copy Info.plist
cp Info.plist "exports/macos/${APP_NAME}.app/Contents/Info.plist"
# copy the icon (assuming you already have it in Apple ICNS format)
cp AppIcon.icns "exports/macos/${APP_NAME}.app/Contents/Resources/AppIcon.icns"
# copy your Bevy game assets
cp -a assets "exports/macos/${APP_NAME}.app/Contents/MacOS/"
# compile the executables for each architecture
cargo build --release --target x86_64-apple-darwin # build for Intel
cargo build --release --target aarch64-apple-darwin # build for Apple Silicon
# combine the executables into a single file and put it in the bundle
lipo "target/x86_64-apple-darwin/release/${RUST_CRATE_NAME}" \
     "target/aarch64-apple-darwin/release/${RUST_CRATE_NAME}" \
     -create -output "exports/macos/${APP_NAME}.app/Contents/MacOS/${APP_NAME}"
