trace("// array = new Array(1)");
array = new Array(1);
trace(array);
trace(array.length);
trace("");

trace("// array[0] = \"a\"");
array[0] = "a";
trace(array);
trace(array.length);
trace("");

trace("// array[1] = \"b\"");
array[1] = "b";
trace(array);
trace(array.length);
trace("");

trace("// array[-1] = \"irrelevant\"");
array[-1] = "irrelevant";
trace(array);
trace(array.length);
trace("");

trace("// array[\"foo\"] = \"irrelephant\"");
array["foo"] = "irrelephant";
trace(array);
trace(array.length);
trace("");

trace("// array[1] = \"b\"");
array[4] = "undefined";
trace(array);
trace(array.length);
trace("");

trace("// array[5] = \"c\"");
array[5] = "c";
trace(array);
trace(array.length);
trace("");

trace("// array.hasOwnProperty(0)");
trace(array.hasOwnProperty(0));
trace("");

trace("// array.hasOwnProperty(3)");
trace(array.hasOwnProperty(3));
trace("");

trace("// array.hasOwnProperty(4)");
trace(array.hasOwnProperty(4));
trace("");

fscommand("quit");
