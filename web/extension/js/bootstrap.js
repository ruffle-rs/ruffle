// Browser extensions are loaded from a dynamically-generated URL, we have to
// tell webpack about that.

__webpack_public_path__ = browser.runtime.getURL("dist/0.ruffle.js").replace("0.ruffle.js", "");

// A dependency graph that contains any wasm must all be imported
// asynchronously. This `bootstrap.js` file does the single async import, so
// that no one else needs to worry about it again.
import("./index.js")
  .catch(e => console.error("Error importing `index.js`:", e));
