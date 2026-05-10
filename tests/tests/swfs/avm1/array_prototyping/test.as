base = ["a","b","c"];
array = {};
array.length = 3;

array.addProperty(2, function(){return "x";}, null);

array.__proto__ = base;
trace("// array");
trace(array);
trace("// array[0]");
trace(array[0]);
trace("// array[1]");
trace(array[1]);
trace("// array[2]");
trace(array[2]);
trace("// array[3]");
trace(array[3]);
trace("// array.length");
trace(array.length);

fscommand("quit");
