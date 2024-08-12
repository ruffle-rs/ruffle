#!/bin/bash

# Package the macOS executable binary into a Ruffle app bundle inside a DMG image

# Input arguments
# The path of the macOS executable binary
EXECUTABLE_BINARY_PATH=$1
# The path of the Safari web extension binary
EXTENSION_BINARY_PATH=$2
# The path of the Safari web extension zip file
EXTENSION_ZIP_PATH=$3
# Whether this binary can run on Intel chips
RUNS_ON_INTEL=$4
# The minimum macOS version this binary can run on
# (This can be calculated with RUNS_ON_INTEL, but it's given separately to avoid duplication)
MIN_MACOS_VERSION=$5
# The path the DMG file should be written onto
DMG_PATH=$6
# The environment variables APPLE_DEVELOPER_KEY, APPLE_DEVELOPER_KEY_PASSWORD, APPLE_DEVELOPER_IDENTITY, APPLE_ID,
# APPLE_TEAM and APPLE_APP_PASSWORD are also required

# Echos the input text in red to stderr
echo_warning() {
	# Color codes for error messages
	RED='\033[0;31m'
	RESET='\033[0m'

	echo -e "\n${RED}$1${RESET}\n" >&2
}

# Bash configuration
set -e

# Create the bundle
echo "Create the bundle"
VERSION=$(yq eval '.workspace.package.version' Cargo.toml)
sed -i "" "s/{Version}/$VERSION/" desktop/packaging/macOS/Info.plist
sed -i "" "s/{MinimumMacOSVersion}/$MIN_MACOS_VERSION/" desktop/packaging/macOS/Info.plist
sed -i "" "s/{CurrentYear}/$(date +%Y)/" desktop/packaging/macOS/Info.plist
mkdir -p package/Ruffle.app/Contents
mkdir package/Ruffle.app/Contents/MacOS
mkdir package/Ruffle.app/Contents/Resources
cp desktop/packaging/macOS/Info.plist package/Ruffle.app/Contents/Info.plist
cp $EXECUTABLE_BINARY_PATH package/Ruffle.app/Contents/MacOS/ruffle
cp common_package/* package/Ruffle.app/Contents/Resources

# Compile the asset catalog
echo -e "\nCompile the asset catalog"
xcrun actool --compile package/Ruffle.app/Contents/Resources desktop/packaging/macOS/Assets.xcassets \
	--minimum-deployment-target $MIN_MACOS_VERSION --platform macosx --warnings --errors --notices --include-all-app-icons

# Create the extension bundle
echo -e "\nCreate the extension bundle"
(
	mkdir -p package/Ruffle.app/Contents/PlugIns/Ruffle\ Web.appex/Contents
	mkdir package/Ruffle.app/Contents/PlugIns/Ruffle\ Web.appex/Contents/Resources
	mkdir package/Ruffle.app/Contents/PlugIns/Ruffle\ Web.appex/Contents/MacOS
	cp web/packages/extension/safari/packaging/Info.plist package/Ruffle.app/Contents/PlugIns/Ruffle\ Web.appex/Contents/Info.plist
	cp $EXTENSION_BINARY_PATH package/Ruffle.app/Contents/PlugIns/Ruffle\ Web.appex/Contents/MacOS/
	unzip $EXTENSION_ZIP_PATH -d package/Ruffle.app/Contents/PlugIns/Ruffle\ Web.appex/Contents/Resources/
) || (
	echo_warning "Could not create the extension bundle"
)


# Sign the app bundle
echo -e "\nSign the app bundle"
(
	echo $APPLE_DEVELOPER_KEY | base64 --decode > certificate.p12
	security create-keychain -p correct-horse-battery-staple build.keychain
	security default-keychain -s build.keychain
	security unlock-keychain -p correct-horse-battery-staple build.keychain
	security import certificate.p12 -k build.keychain -P $APPLE_DEVELOPER_KEY_PASSWORD -T /usr/bin/codesign
	security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k correct-horse-battery-staple build.keychain
	codesign --deep -s $APPLE_DEVELOPER_IDENTITY -f --entitlements desktop/packaging/macOS/Entitlements.plist \
		--options runtime package/Ruffle.app
	codesign --verify -vvvv package/Ruffle.app
) || (
	echo_warning "Could not sign the app bundle"
)


# Notarize the app bundle
echo -e "\nNotarize the app bundle"
(
	xcrun notarytool store-credentials "Ruffle" --apple-id $APPLE_ID --team-id $APPLE_TEAM --password $APPLE_APP_PASSWORD
	cd package
	zip -r Ruffle.zip Ruffle.app
	mv Ruffle.zip ..
	cd ..
	xcrun notarytool submit Ruffle.zip --keychain-profile Ruffle --wait
	xcrun stapler staple package/Ruffle.app
) || (
	echo_warning "Could not notarize the app bundle"
)


# TODO: Sign, notarize and staple the DMG image (instead of the app bundle)
# Package the macOS app bundle as DMG image
echo -e "\nPackage the macOS app bundle as DMG image"
pip install dmgbuild
if [[ "$RUNS_ON_INTEL" == "true" ]]; then
	FORMAT="UDZO"
	FILESYSTEM="HFS+"
else
	FORMAT="ULMO"
	FILESYSTEM="APFS"
fi
dmgbuild -s desktop/packaging/macOS/settings.py "Ruffle" -D format=$FORMAT -D filesystem=$FILESYSTEM $DMG_PATH
