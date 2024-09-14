To update to a new OpenH264 release:

1. Open the new release on https://github.com/cisco/openh264/releases
2. Update the binary file names and SHA256 hashes (see below)
3. Add/remove supported architectures and platforms if added/dropped
4. Update the base URL and license if changed
5. Download the OpenH264 sources (at least `codec_api.h`)
6. Regenerate the bindings using the command in `decoder.rs`
7. Follow the API changes if necessary
8. Update the version number sanity check

Cisco provides only binaries and MD5 hashes over HTTP which is far from being secure.
We want to use SHA256 in order to protect users from downloading malicious binaries.
Currently, we have to calculate hashes ourselves from the downloaded binaries:

1. Make sure you're using a secure network
2. Download necessary libraries and unpack them
3. Verify their MD5 checksums
4. Calculate their SHA256 checksums
5. If you're unsure, repeat this over a different network
