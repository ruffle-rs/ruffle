
# Prerequisite

Install Android Sudio with at least the Platform SDK (e.g. 28) and the NDK Tools.

Also:

`cargo install cargo-apk`

# Build Steps

NOTE: First a sacrificial APK is built, then the native library it produces is used to build the proper APK.

Substitute the appropriate locations and ndk version in the variables set for the `cargo-apk` command.

```bash
cd native
ANDROID_SDK_ROOT=$HOME/Android/Sdk/ ANDROID_NDK_ROOT=$HOME/Android/Sdk/ndk/24.0.8215888/ cargo apk build --release
cd ..

cp -r ../target/release/apk/lib/arm64-v8a app/ruffle/src/main/jniLibs/

cd app
./gradlew build
cd ..
```

The final APK should be at:

`ruffle/android/app/ruffle/build/outputs/apk/release/ruffle-release-unsigned.apk`

After the first step, simply opening the `app` project in Android Studio for development also works.

---

32-bit ARM and x86_64 support is TBD.