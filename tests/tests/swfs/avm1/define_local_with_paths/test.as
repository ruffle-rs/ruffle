// SWF hand-edited with JPEXS.
// This can also be done in the Flash editor, when switching the ActionScript version to ActionScript 1.0 in File -> Publish Properties.

function /:f1()
{
   trace("// Inside function");
   trace("");
   trace("// ghi - defined outside function");
   trace(ghi + /:ghi);
   trace("");
   var _root.v123 = "123";
   trace("// var _root.v123 = \'123\'");
   trace(_root.v123 + /:v123);
   trace("");
   var _loc2_ = {};
   with(_loc2_)
   {
      trace("// Inside function -> with({})");
      trace("");
      var /:v456 = "456";
      trace("// var /:v456 = \'456\'");
      trace(/:v456 + _root.v456);
      trace("");
      
   };
   tellTarget("_root")
   {
      trace("// Inside function -> tellTarget(_root)");
      var /:v789 = "789";
      trace("// var /:v789 = \'789\'");
      trace(/:v789 + _root.v789);
      trace("");
   }
   var _root.mno;
}
function f2()
{
   var /:pqr = "PQR";
   var g = g;
   g();
}
function g()
{
   trace("// var /:pqr = \'PQR\';\n// this[\'/:pqr\']");
   trace(this["/:pqr"]);
   trace("");
}
var /:abc = "ABC";
trace("// var /:abc = \'ABC\'");
trace(/:abc + _root.abc);
trace("");
var /ruffle/:def = "DEF";
trace("// var /ruffle/:def = \'DEF\'");
trace(/ruffle/:def + _root.ruffle.def);
trace("");
_root.ruffle = {};
trace("_root.ruffle = {};");
trace("");
var /ruffle/:def = "DEF";
trace("// var /ruffle/:def = \'DEF\'");
trace(/ruffle/:def + _root.ruffle.def);
trace("");
var abc = "XYZ";
trace("// var abc = \'XYZ\';\n// /:abc + abc");
trace(/:abc + abc);
trace("");
var /:ghi;
trace("// var /:ghi;\n// _root.hasOwnProperty(\'ghi\')");
trace(_root.hasOwnProperty("ghi"));
trace("");
var _root.ghi = "GHI";
var /:ghi;
trace("// var _root.ghi = \'GHI\';\n// var /:ghi");
trace(/:ghi + _root.ghi);
trace("");
var _root.o = {};
with(_root.o)
{
   trace("// Inside with({})");
   trace("");
   var /:jkl = "JKL";
   trace("// var /:jkl = \'JKL\'");
   trace(/:jkl + _root.jkl);
   trace("");
   
};
f1();
trace("// Outside function - tracing all variables defined inside function");
trace(/:v123 + _root.v456 + _root.v789);
trace("");
trace("// _root.hasOwnProperty(\'mno\') - defined inside function");
trace(_root.hasOwnProperty("mno"));
trace("");
f2();
