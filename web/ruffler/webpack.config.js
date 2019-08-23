'use strict';

const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const path = require('path');

class EmitExtensionManifestPlugin {
  apply(compiler) {
    compiler.hooks.emit.tapAsync('Emit Extension Manifest', (compilation, callback) => {
      const manifest = require('./manifest.json');
      Object.keys(compilation.assets).forEach((name) => {
        manifest.web_accessable_resources.push(`./${name}`);
      });
      manifest.content_scripts[0].js.push(`./${compiler.options.output.filename}`);
      const source = JSON.stringify(manifest, null, 2);
      compilation.assets['manifest.json'] = {
        source: () => source,
        size: () => source.length,
      };
      callback();
    });
  }
}

module.exports = () => ({
  entry: './src/index.mjs',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'ruffler.js',
  },
  plugins: [
    new CleanWebpackPlugin(),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '..'),
    }),
    new EmitExtensionManifestPlugin(),
  ],
});
