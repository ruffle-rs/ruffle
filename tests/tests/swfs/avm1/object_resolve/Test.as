// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf -version 8
class Test {

  static function main(current) {
    var object = {};
    var values = {};

    values["string"] = "A String!";
    values["undefined"] = undefined;
    values["object"] = {};
    values["function"] = function() {
      trace("// function called!");
      if (this === object) {
        trace("// this == object");
      } else if (this === values) {
        trace("// this === values");
      } else {
        trace("// this == " + this);
      }
    };

    object.__resolve = function(name) {
      trace("// __resolve(" + name + ") called!");
      return values[name];
    };

    trace("object.string: " + object.string);
    trace("object.undefined: " + object.undefined);
    trace("object.object: " + object.object);
    trace("object.function: " + object["function"]);
    trace("object.function(): " + object["function"]());

    trace("object.hasOwnProperty(\"function\"): " + object.hasOwnProperty("function"));

    trace("");

    var Proto = function() {};
    Proto.prototype.__resolve = function(name) {
      trace("// Proto.prototype.__resolve function called!");
      return values[name];
    };
    object = new Proto();
    object.string = "An overriden string!";
    Proto.prototype.onThePrototype = "This was on the prototype!";
    trace("object.string: " + object.string);
    trace("object.onThePrototype: " + object.onThePrototype);
    trace("object.undefined: " + object.undefined);
    trace("object.object: " + object.object);
    trace("object.function: " + object["function"]);
    trace("object.function(): " + object["function"]());

    trace("object.hasOwnProperty(\"function\"): " + object.hasOwnProperty("function"));

    trace("");

    trace("// object.__resolve = 42");
    object.__resolve = 42;
    trace("object.object: " + object.object);
    trace("// object.__resolve = _root");
    object.__resolve = _root;
    trace("object.object: " + object.object);
    trace("// object.__resolve = {}");
    object.__resolve = {};
    trace("object.object: " + object.object);

    trace("");

    object = {};
    object.addProperty("__resolve", function() {return function() {return "resolved!";}}, null);
    trace("object.foo with an addProperty __resolve: " + object.foo);
  }
}