var array = new Array(16);
array[1] = undefined;
array[0] = "elem 0";
array.foo = "foo";
array[5] = "elem 5";

for(var i in array)
{
	trace("array[" + i + "]: " + array[i]);
}
trace("");

fscommand("quit");
