// Regression test for #24499.
// ASSetPropFlags(obj, null, 0, 1) clears DontEnum on every property, which in
// Flash exposes `constructor` to for..in alongside `__proto__`. Ruffle only
// exposed `__proto__`.

var obj = {};
ASSetPropFlags(obj, null, 0, 1);
for (var key in obj) {
    trace(key);
}
trace(obj.constructor);
