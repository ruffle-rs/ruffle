Object.assign(exports, require("./public-api"));
Object.assign(exports, require("./plugin-polyfill"));
Object.assign(exports, require("./public-path"));
Object.assign(exports, require("./polyfills"));
Object.assign(exports, require("./register-element"));
Object.assign(exports, require("./ruffle-embed"));
Object.assign(exports, require("./ruffle-imports"));
Object.assign(exports, require("./ruffle-player"));
Object.assign(exports, require("./source-api"));
Object.assign(exports, require("./version-range"));
Object.assign(exports, require("./version"));

exports.load_ruffle = require("./load-ruffle");
exports.RuffleObject = require("./ruffle-object");
exports.ruffle_shadow_template = require("./shadow-template");
