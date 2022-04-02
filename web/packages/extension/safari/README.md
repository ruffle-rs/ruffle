This directory builds the native half of Ruffle's Safari Web Extension for
macOS.

The binary this crate produces is intended to be bundled into an `.appex`
package along with the contents of the `packages/macOS` directory, in the same
way that the Desktop binary is packaged for macOS. The compiled web extension
itself should be placed in the package's `Contents/Resources` directory.