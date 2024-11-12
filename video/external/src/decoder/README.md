To update to a new OpenH264 release:

1. Open the new release on https://github.com/cisco/openh264/releases
2. Update the binary file names and SHA256 hashes
3. Add/remove supported architectures and platforms if added/dropped
4. Update the base URL and license if changed
5. Download the OpenH264 sources (at least `codec_api.h`)
6. Regenerate the bindings using the command in `decoder.rs`
7. Follow the API changes if necessary
8. Update the version number sanity check
