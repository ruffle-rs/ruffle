var root_var = 5;
var obj_var = {};
trace(a);
trace(b);
trace(child);
trace(root_var);
trace(typeof root_var);
trace(typeof obj_var);
trace("path /");
trace("  _x:" add getProperty("/", _x));
trace("  _name:" add getProperty("/", _name));
trace("  typeof eval:" add (typeof eval("/")));
tellTarget("/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path //");
trace("  _x:" add getProperty("//", _x));
trace("  _name:" add getProperty("//", _name));
trace("  typeof eval:" add (typeof eval("//")));
tellTarget("//") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a");
trace("  _x:" add getProperty("/a", _x));
trace("  _name:" add getProperty("/a", _name));
trace("  typeof eval:" add (typeof eval("/a")));
tellTarget("/a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path //a");
trace("  _x:" add getProperty("//a", _x));
trace("  _name:" add getProperty("//a", _name));
trace("  typeof eval:" add (typeof eval("//a")));
tellTarget("//a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /");
trace("  _x:" add getProperty("/", _x));
trace("  _name:" add getProperty("/", _name));
trace("  typeof eval:" add (typeof eval("/")));
tellTarget("/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a");
trace("  _x:" add getProperty("/a", _x));
trace("  _name:" add getProperty("/a", _name));
trace("  typeof eval:" add (typeof eval("/a")));
tellTarget("/a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a/");
trace("  _x:" add getProperty("/a/", _x));
trace("  _name:" add getProperty("/a/", _name));
trace("  typeof eval:" add (typeof eval("/a/")));
tellTarget("/a/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a");
trace("  _x:" add getProperty("a", _x));
trace("  _name:" add getProperty("a", _name));
trace("  typeof eval:" add (typeof eval("a")));
tellTarget("a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/");
trace("  _x:" add getProperty("a/", _x));
trace("  _name:" add getProperty("a/", _name));
trace("  typeof eval:" add (typeof eval("a/")));
tellTarget("a/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/..");
trace("  _x:" add getProperty("a/..", _x));
trace("  _name:" add getProperty("a/..", _name));
trace("  typeof eval:" add (typeof eval("a/..")));
tellTarget("a/..") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/../");
trace("  _x:" add getProperty("a/../", _x));
trace("  _name:" add getProperty("a/../", _name));
trace("  typeof eval:" add (typeof eval("a/../")));
tellTarget("a/../") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/..:");
trace("  _x:" add getProperty("a/..:", _x));
trace("  _name:" add getProperty("a/..:", _name));
trace("  typeof eval:" add (typeof eval("a/..:")));
tellTarget("a/..:") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/../a");
trace("  _x:" add getProperty("a/../a", _x));
trace("  _name:" add getProperty("a/../a", _name));
trace("  typeof eval:" add (typeof eval("a/../a")));
tellTarget("a/../a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/..:a");
trace("  _x:" add getProperty("a/..:a", _x));
trace("  _name:" add getProperty("a/..:a", _name));
trace("  typeof eval:" add (typeof eval("a/..:a")));
tellTarget("a/..:a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path ../a");
trace("  _x:" add getProperty("../a", _x));
trace("  _name:" add getProperty("../a", _name));
trace("  typeof eval:" add (typeof eval("../a")));
tellTarget("../a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/a/..");
trace("  _x:" add getProperty("a/a/..", _x));
trace("  _name:" add getProperty("a/a/..", _name));
trace("  typeof eval:" add (typeof eval("a/a/..")));
tellTarget("a/a/..") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/a/../");
trace("  _x:" add getProperty("a/a/../", _x));
trace("  _name:" add getProperty("a/a/../", _name));
trace("  typeof eval:" add (typeof eval("a/a/../")));
tellTarget("a/a/../") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/a/../..");
trace("  _x:" add getProperty("a/a/../..", _x));
trace("  _name:" add getProperty("a/a/../..", _name));
trace("  typeof eval:" add (typeof eval("a/a/../..")));
tellTarget("a/a/../..") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/a/../../");
trace("  _x:" add getProperty("a/a/../../", _x));
trace("  _name:" add getProperty("a/a/../../", _name));
trace("  typeof eval:" add (typeof eval("a/a/../../")));
tellTarget("a/a/../../") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/a/../../a");
trace("  _x:" add getProperty("a/a/../../a", _x));
trace("  _name:" add getProperty("a/a/../../a", _name));
trace("  typeof eval:" add (typeof eval("a/a/../../a")));
tellTarget("a/a/../../a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path variable");
trace("  _x:" add getProperty("variable", _x));
trace("  _name:" add getProperty("variable", _name));
trace("  typeof eval:" add (typeof eval("variable")));
tellTarget("variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /:variable");
trace("  _x:" add getProperty("/:variable", _x));
trace("  _name:" add getProperty("/:variable", _name));
trace("  typeof eval:" add (typeof eval("/:variable")));
tellTarget("/:variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path :variable");
trace("  _x:" add getProperty(":variable", _x));
trace("  _name:" add getProperty(":variable", _name));
trace("  typeof eval:" add (typeof eval(":variable")));
tellTarget(":variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a:variable");
trace("  _x:" add getProperty("/a:variable", _x));
trace("  _name:" add getProperty("/a:variable", _name));
trace("  typeof eval:" add (typeof eval("/a:variable")));
tellTarget("/a:variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a:variable");
trace("  _x:" add getProperty("a:variable", _x));
trace("  _name:" add getProperty("a:variable", _name));
trace("  typeof eval:" add (typeof eval("a:variable")));
tellTarget("a:variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a/:variable");
trace("  _x:" add getProperty("/a/:variable", _x));
trace("  _name:" add getProperty("/a/:variable", _name));
trace("  typeof eval:" add (typeof eval("/a/:variable")));
tellTarget("/a/:variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/:variable");
trace("  _x:" add getProperty("a/:variable", _x));
trace("  _name:" add getProperty("a/:variable", _name));
trace("  typeof eval:" add (typeof eval("a/:variable")));
tellTarget("a/:variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/../a:variable");
trace("  _x:" add getProperty("a/../a:variable", _x));
trace("  _name:" add getProperty("a/../a:variable", _name));
trace("  typeof eval:" add (typeof eval("a/../a:variable")));
tellTarget("a/../a:variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /:root_var");
trace("  _x:" add getProperty("/:root_var", _x));
trace("  _name:" add getProperty("/:root_var", _name));
trace("  typeof eval:" add (typeof eval("/:root_var")));
tellTarget("/:root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path root_var");
trace("  _x:" add getProperty("root_var", _x));
trace("  _name:" add getProperty("root_var", _name));
trace("  typeof eval:" add (typeof eval("root_var")));
tellTarget("root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path :root_var");
trace("  _x:" add getProperty(":root_var", _x));
trace("  _name:" add getProperty(":root_var", _name));
trace("  typeof eval:" add (typeof eval(":root_var")));
tellTarget(":root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a:root_var");
trace("  _x:" add getProperty("/a:root_var", _x));
trace("  _name:" add getProperty("/a:root_var", _name));
trace("  typeof eval:" add (typeof eval("/a:root_var")));
tellTarget("/a:root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a:root_var");
trace("  _x:" add getProperty("a:root_var", _x));
trace("  _name:" add getProperty("a:root_var", _name));
trace("  typeof eval:" add (typeof eval("a:root_var")));
tellTarget("a:root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a/:root_var");
trace("  _x:" add getProperty("/a/:root_var", _x));
trace("  _name:" add getProperty("/a/:root_var", _name));
trace("  typeof eval:" add (typeof eval("/a/:root_var")));
tellTarget("/a/:root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/:root_var");
trace("  _x:" add getProperty("a/:root_var", _x));
trace("  _name:" add getProperty("a/:root_var", _name));
trace("  typeof eval:" add (typeof eval("a/:root_var")));
tellTarget("a/:root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/../a:root_var");
trace("  _x:" add getProperty("a/../a:root_var", _x));
trace("  _name:" add getProperty("a/../a:root_var", _name));
trace("  typeof eval:" add (typeof eval("a/../a:root_var")));
tellTarget("a/../a:root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /:obj_var");
trace("  _x:" add getProperty("/:obj_var", _x));
trace("  _name:" add getProperty("/:obj_var", _name));
trace("  typeof eval:" add (typeof eval("/:obj_var")));
tellTarget("/:obj_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /.variable");
trace("  _x:" add getProperty("/.variable", _x));
trace("  _name:" add getProperty("/.variable", _name));
trace("  typeof eval:" add (typeof eval("/.variable")));
tellTarget("/.variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path .variable");
trace("  _x:" add getProperty(".variable", _x));
trace("  _name:" add getProperty(".variable", _name));
trace("  typeof eval:" add (typeof eval(".variable")));
tellTarget(".variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a.variable");
trace("  _x:" add getProperty("/a.variable", _x));
trace("  _name:" add getProperty("/a.variable", _name));
trace("  typeof eval:" add (typeof eval("/a.variable")));
tellTarget("/a.variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a.variable");
trace("  _x:" add getProperty("a.variable", _x));
trace("  _name:" add getProperty("a.variable", _name));
trace("  typeof eval:" add (typeof eval("a.variable")));
tellTarget("a.variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a/.variable");
trace("  _x:" add getProperty("/a/.variable", _x));
trace("  _name:" add getProperty("/a/.variable", _name));
trace("  typeof eval:" add (typeof eval("/a/.variable")));
tellTarget("/a/.variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/.variable");
trace("  _x:" add getProperty("a/.variable", _x));
trace("  _name:" add getProperty("a/.variable", _name));
trace("  typeof eval:" add (typeof eval("a/.variable")));
tellTarget("a/.variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/../a.variable");
trace("  _x:" add getProperty("a/../a.variable", _x));
trace("  _name:" add getProperty("a/../a.variable", _name));
trace("  typeof eval:" add (typeof eval("a/../a.variable")));
tellTarget("a/../a.variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /.root_var");
trace("  _x:" add getProperty("/.root_var", _x));
trace("  _name:" add getProperty("/.root_var", _name));
trace("  typeof eval:" add (typeof eval("/.root_var")));
tellTarget("/.root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path .root_var");
trace("  _x:" add getProperty(".root_var", _x));
trace("  _name:" add getProperty(".root_var", _name));
trace("  typeof eval:" add (typeof eval(".root_var")));
tellTarget(".root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a.root_var");
trace("  _x:" add getProperty("/a.root_var", _x));
trace("  _name:" add getProperty("/a.root_var", _name));
trace("  typeof eval:" add (typeof eval("/a.root_var")));
tellTarget("/a.root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a.root_var");
trace("  _x:" add getProperty("a.root_var", _x));
trace("  _name:" add getProperty("a.root_var", _name));
trace("  typeof eval:" add (typeof eval("a.root_var")));
tellTarget("a.root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /a/.root_var");
trace("  _x:" add getProperty("/a/.root_var", _x));
trace("  _name:" add getProperty("/a/.root_var", _name));
trace("  typeof eval:" add (typeof eval("/a/.root_var")));
tellTarget("/a/.root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/.root_var");
trace("  _x:" add getProperty("a/.root_var", _x));
trace("  _name:" add getProperty("a/.root_var", _name));
trace("  typeof eval:" add (typeof eval("a/.root_var")));
tellTarget("a/.root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/../a.root_var");
trace("  _x:" add getProperty("a/../a.root_var", _x));
trace("  _name:" add getProperty("a/../a.root_var", _name));
trace("  typeof eval:" add (typeof eval("a/../a.root_var")));
tellTarget("a/../a.root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0:root_var");
trace("  _x:" add getProperty("_level0:root_var", _x));
trace("  _name:" add getProperty("_level0:root_var", _name));
trace("  typeof eval:" add (typeof eval("_level0:root_var")));
tellTarget("_level0:root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0:variable");
trace("  _x:" add getProperty("_level0:variable", _x));
trace("  _name:" add getProperty("_level0:variable", _name));
trace("  typeof eval:" add (typeof eval("_level0:variable")));
tellTarget("_level0:variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:root_var");
trace("  _x:" add getProperty("_level0/:root_var", _x));
trace("  _name:" add getProperty("_level0/:root_var", _name));
trace("  typeof eval:" add (typeof eval("_level0/:root_var")));
tellTarget("_level0/:root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:variable");
trace("  _x:" add getProperty("_level0/:variable", _x));
trace("  _name:" add getProperty("_level0/:variable", _name));
trace("  typeof eval:" add (typeof eval("_level0/:variable")));
tellTarget("_level0/:variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0.root_var");
trace("  _x:" add getProperty("_level0.root_var", _x));
trace("  _name:" add getProperty("_level0.root_var", _name));
trace("  typeof eval:" add (typeof eval("_level0.root_var")));
tellTarget("_level0.root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0.variable");
trace("  _x:" add getProperty("_level0.variable", _x));
trace("  _name:" add getProperty("_level0.variable", _name));
trace("  typeof eval:" add (typeof eval("_level0.variable")));
tellTarget("_level0.variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/.root_var");
trace("  _x:" add getProperty("_level0/.root_var", _x));
trace("  _name:" add getProperty("_level0/.root_var", _name));
trace("  typeof eval:" add (typeof eval("_level0/.root_var")));
tellTarget("_level0/.root_var") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/.variable");
trace("  _x:" add getProperty("_level0/.variable", _x));
trace("  _name:" add getProperty("_level0/.variable", _name));
trace("  typeof eval:" add (typeof eval("_level0/.variable")));
tellTarget("_level0/.variable") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/child");
trace("  _x:" add getProperty("a/child", _x));
trace("  _name:" add getProperty("a/child", _name));
trace("  typeof eval:" add (typeof eval("a/child")));
tellTarget("a/child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a:child");
trace("  _x:" add getProperty("a:child", _x));
trace("  _name:" add getProperty("a:child", _name));
trace("  typeof eval:" add (typeof eval("a:child")));
tellTarget("a:child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a.child");
trace("  _x:" add getProperty("a.child", _x));
trace("  _name:" add getProperty("a.child", _name));
trace("  typeof eval:" add (typeof eval("a.child")));
tellTarget("a.child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0.a/child");
trace("  _x:" add getProperty("_level0.a/child", _x));
trace("  _name:" add getProperty("_level0.a/child", _name));
trace("  typeof eval:" add (typeof eval("_level0.a/child")));
tellTarget("_level0.a/child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/a.child");
trace("  _x:" add getProperty("_level0/a.child", _x));
trace("  _name:" add getProperty("_level0/a.child", _name));
trace("  typeof eval:" add (typeof eval("_level0/a.child")));
tellTarget("_level0/a.child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0:a:child");
trace("  _x:" add getProperty("_level0:a:child", _x));
trace("  _name:" add getProperty("_level0:a:child", _name));
trace("  typeof eval:" add (typeof eval("_level0:a:child")));
tellTarget("_level0:a:child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0.a:child");
trace("  _x:" add getProperty("_level0.a:child", _x));
trace("  _name:" add getProperty("_level0.a:child", _name));
trace("  typeof eval:" add (typeof eval("_level0.a:child")));
tellTarget("_level0.a:child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0:a.child");
trace("  _x:" add getProperty("_level0:a.child", _x));
trace("  _name:" add getProperty("_level0:a.child", _name));
trace("  typeof eval:" add (typeof eval("_level0:a.child")));
tellTarget("_level0:a.child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0:a/child");
trace("  _x:" add getProperty("_level0:a/child", _x));
trace("  _name:" add getProperty("_level0:a/child", _name));
trace("  typeof eval:" add (typeof eval("_level0:a/child")));
tellTarget("_level0:a/child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/a:child");
trace("  _x:" add getProperty("_level0/a:child", _x));
trace("  _name:" add getProperty("_level0/a:child", _name));
trace("  typeof eval:" add (typeof eval("_level0/a:child")));
tellTarget("_level0/a:child") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /_level0");
trace("  _x:" add getProperty("/_level0", _x));
trace("  _name:" add getProperty("/_level0", _name));
trace("  typeof eval:" add (typeof eval("/_level0")));
tellTarget("/_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0");
trace("  _x:" add getProperty("_level0", _x));
trace("  _name:" add getProperty("_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0")));
tellTarget("_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/_level0");
trace("  _x:" add getProperty("_level0/_level0", _x));
trace("  _name:" add getProperty("_level0/_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0/_level0")));
tellTarget("_level0/_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path unknown/_level0");
trace("  _x:" add getProperty("unknown/_level0", _x));
trace("  _name:" add getProperty("unknown/_level0", _name));
trace("  typeof eval:" add (typeof eval("unknown/_level0")));
tellTarget("unknown/_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/a:_level0");
trace("  _x:" add getProperty("_level0/a:_level0", _x));
trace("  _name:" add getProperty("_level0/a:_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0/a:_level0")));
tellTarget("_level0/a:_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/a");
trace("  _x:" add getProperty("_level0/a", _x));
trace("  _name:" add getProperty("_level0/a", _name));
trace("  typeof eval:" add (typeof eval("_level0/a")));
tellTarget("_level0/a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/../a");
trace("  _x:" add getProperty("_level0/../a", _x));
trace("  _name:" add getProperty("_level0/../a", _name));
trace("  typeof eval:" add (typeof eval("_level0/../a")));
tellTarget("_level0/../a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/../_level0");
trace("  _x:" add getProperty("_level0/../_level0", _x));
trace("  _name:" add getProperty("_level0/../_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0/../_level0")));
tellTarget("_level0/../_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:a/");
trace("  _x:" add getProperty("_level0/:a/", _x));
trace("  _name:" add getProperty("_level0/:a/", _name));
trace("  typeof eval:" add (typeof eval("_level0/:a/")));
tellTarget("_level0/:a/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/.a/");
trace("  _x:" add getProperty("_level0/.a/", _x));
trace("  _name:" add getProperty("_level0/.a/", _name));
trace("  typeof eval:" add (typeof eval("_level0/.a/")));
tellTarget("_level0/.a/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:a/_level0");
trace("  _x:" add getProperty("_level0/:a/_level0", _x));
trace("  _name:" add getProperty("_level0/:a/_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0/:a/_level0")));
tellTarget("_level0/:a/_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:a:_level0");
trace("  _x:" add getProperty("_level0/:a:_level0", _x));
trace("  _name:" add getProperty("_level0/:a:_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0/:a:_level0")));
tellTarget("_level0/:a:_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:a:");
trace("  _x:" add getProperty("_level0/:a:", _x));
trace("  _name:" add getProperty("_level0/:a:", _name));
trace("  typeof eval:" add (typeof eval("_level0/:a:")));
tellTarget("_level0/:a:") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/.a:");
trace("  _x:" add getProperty("_level0/.a:", _x));
trace("  _name:" add getProperty("_level0/.a:", _name));
trace("  typeof eval:" add (typeof eval("_level0/.a:")));
tellTarget("_level0/.a:") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:a");
trace("  _x:" add getProperty("_level0/:a", _x));
trace("  _name:" add getProperty("_level0/:a", _name));
trace("  typeof eval:" add (typeof eval("_level0/:a")));
tellTarget("_level0/:a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/.a");
trace("  _x:" add getProperty("_level0/.a", _x));
trace("  _name:" add getProperty("_level0/.a", _name));
trace("  typeof eval:" add (typeof eval("_level0/.a")));
tellTarget("_level0/.a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/::a");
trace("  _x:" add getProperty("_level0/::a", _x));
trace("  _name:" add getProperty("_level0/::a", _name));
trace("  typeof eval:" add (typeof eval("_level0/::a")));
tellTarget("_level0/::a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:::a");
trace("  _x:" add getProperty("_level0/:::a", _x));
trace("  _name:" add getProperty("_level0/:::a", _name));
trace("  typeof eval:" add (typeof eval("_level0/:::a")));
tellTarget("_level0/:::a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:::_level0");
trace("  _x:" add getProperty("_level0/:::_level0", _x));
trace("  _name:" add getProperty("_level0/:::_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0/:::_level0")));
tellTarget("_level0/:::_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/::_level0");
trace("  _x:" add getProperty("_level0/::_level0", _x));
trace("  _name:" add getProperty("_level0/::_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0/::_level0")));
tellTarget("_level0/::_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0/:_level0");
trace("  _x:" add getProperty("_level0/:_level0", _x));
trace("  _name:" add getProperty("_level0/:_level0", _name));
trace("  typeof eval:" add (typeof eval("_level0/:_level0")));
tellTarget("_level0/:_level0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /this");
trace("  _x:" add getProperty("/this", _x));
trace("  _name:" add getProperty("/this", _name));
trace("  typeof eval:" add (typeof eval("/this")));
tellTarget("/this") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path this");
trace("  _x:" add getProperty("this", _x));
trace("  _name:" add getProperty("this", _name));
trace("  typeof eval:" add (typeof eval("this")));
tellTarget("this") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /_root");
trace("  _x:" add getProperty("/_root", _x));
trace("  _name:" add getProperty("/_root", _name));
trace("  typeof eval:" add (typeof eval("/_root")));
tellTarget("/_root") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _root");
trace("  _x:" add getProperty("_root", _x));
trace("  _name:" add getProperty("_root", _name));
trace("  typeof eval:" add (typeof eval("_root")));
tellTarget("_root") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _root/a");
trace("  _x:" add getProperty("_root/a", _x));
trace("  _name:" add getProperty("_root/a", _name));
trace("  typeof eval:" add (typeof eval("_root/a")));
tellTarget("_root/a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /_level0/a");
trace("  _x:" add getProperty("/_level0/a", _x));
trace("  _name:" add getProperty("/_level0/a", _name));
trace("  typeof eval:" add (typeof eval("/_level0/a")));
tellTarget("/_level0/a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _flash0");
trace("  _x:" add getProperty("_flash0", _x));
trace("  _name:" add getProperty("_flash0", _name));
trace("  typeof eval:" add (typeof eval("_flash0")));
tellTarget("_flash0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _flash0/");
trace("  _x:" add getProperty("_flash0/", _x));
trace("  _name:" add getProperty("_flash0/", _name));
trace("  typeof eval:" add (typeof eval("_flash0/")));
tellTarget("_flash0/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /_flash0");
trace("  _x:" add getProperty("/_flash0", _x));
trace("  _name:" add getProperty("/_flash0", _name));
trace("  typeof eval:" add (typeof eval("/_flash0")));
tellTarget("/_flash0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _flash0/a");
trace("  _x:" add getProperty("_flash0/a", _x));
trace("  _name:" add getProperty("_flash0/a", _name));
trace("  typeof eval:" add (typeof eval("_flash0/a")));
tellTarget("_flash0/a") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _flash0...:");
trace("  _x:" add getProperty("_flash0...:", _x));
trace("  _name:" add getProperty("_flash0...:", _name));
trace("  typeof eval:" add (typeof eval("_flash0...:")));
tellTarget("_flash0...:") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0...");
trace("  _x:" add getProperty("_level0...", _x));
trace("  _name:" add getProperty("_level0...", _name));
trace("  typeof eval:" add (typeof eval("_level0...")));
tellTarget("_level0...") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path :_flash0:/");
trace("  _x:" add getProperty(":_flash0:/", _x));
trace("  _name:" add getProperty(":_flash0:/", _name));
trace("  typeof eval:" add (typeof eval(":_flash0:/")));
tellTarget(":_flash0:/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0:_level0:/");
trace("  _x:" add getProperty("_level0:_level0:/", _x));
trace("  _name:" add getProperty("_level0:_level0:/", _name));
trace("  typeof eval:" add (typeof eval("_level0:_level0:/")));
tellTarget("_level0:_level0:/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _flash0..:");
trace("  _x:" add getProperty("_flash0..:", _x));
trace("  _name:" add getProperty("_flash0..:", _name));
trace("  typeof eval:" add (typeof eval("_flash0..:")));
tellTarget("_flash0..:") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path ::_flash0..");
trace("  _x:" add getProperty("::_flash0..", _x));
trace("  _name:" add getProperty("::_flash0..", _name));
trace("  typeof eval:" add (typeof eval("::_flash0..")));
tellTarget("::_flash0..") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _root/_level0:/");
trace("  _x:" add getProperty("_root/_level0:/", _x));
trace("  _name:" add getProperty("_root/_level0:/", _name));
trace("  typeof eval:" add (typeof eval("_root/_level0:/")));
tellTarget("_root/_level0:/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a:_level0..");
trace("  _x:" add getProperty("a:_level0..", _x));
trace("  _name:" add getProperty("a:_level0..", _name));
trace("  typeof eval:" add (typeof eval("a:_level0..")));
tellTarget("a:_level0..") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0..:_flash0");
trace("  _x:" add getProperty("_level0..:_flash0", _x));
trace("  _name:" add getProperty("_level0..:_flash0", _name));
trace("  typeof eval:" add (typeof eval("_level0..:_flash0")));
tellTarget("_level0..:_flash0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0..:/");
trace("  _x:" add getProperty("_level0..:/", _x));
trace("  _name:" add getProperty("_level0..:/", _name));
trace("  typeof eval:" add (typeof eval("_level0..:/")));
tellTarget("_level0..:/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _root:_flash0..");
trace("  _x:" add getProperty("_root:_flash0..", _x));
trace("  _name:" add getProperty("_root:_flash0..", _name));
trace("  typeof eval:" add (typeof eval("_root:_flash0..")));
tellTarget("_root:_flash0..") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _flash0..");
trace("  _x:" add getProperty("_flash0..", _x));
trace("  _name:" add getProperty("_flash0..", _name));
trace("  typeof eval:" add (typeof eval("_flash0..")));
tellTarget("_flash0..") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _flash0/_flash0:/");
trace("  _x:" add getProperty("_flash0/_flash0:/", _x));
trace("  _name:" add getProperty("_flash0/_flash0:/", _name));
trace("  typeof eval:" add (typeof eval("_flash0/_flash0:/")));
tellTarget("_flash0/_flash0:/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path _level0..:_flash0");
trace("  _x:" add getProperty("_level0..:_flash0", _x));
trace("  _name:" add getProperty("_level0..:_flash0", _name));
trace("  typeof eval:" add (typeof eval("_level0..:_flash0")));
tellTarget("_level0..:_flash0") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path /:_level0:/");
trace("  _x:" add getProperty("/:_level0:/", _x));
trace("  _name:" add getProperty("/:_level0:/", _name));
trace("  typeof eval:" add (typeof eval("/:_level0:/")));
tellTarget("/:_level0:/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
trace("path a/_flash0:/");
trace("  _x:" add getProperty("a/_flash0:/", _x));
trace("  _name:" add getProperty("a/_flash0:/", _name));
trace("  typeof eval:" add (typeof eval("a/_flash0:/")));
tellTarget("a/_flash0:/") {
  trace("  tellTarget");
  trace("  _x:" add _x);
  trace("  _name:" add _name);
}
