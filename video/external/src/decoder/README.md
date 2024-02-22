To update to a new OpenH264 release:

 - Open the new release on https://github.com/cisco/openh264/releases
 - Update the binary file names and MD5 hashes
 - Add/remove supported architectures and platforms if added/dropped
 - Update the base URL and license if changed
 - Download the OpenH264 sources (at least `codec_api.h`)
 - Regenerate the bindings using the command in `decoder.rs`
 - Follow the API changes if necessary
 - Update the version number sanity check
