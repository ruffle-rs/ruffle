var arr:Array = new Array();

arr.length = 10;

// Populate every canonical index except 5. With a short truncation range,
// this exercises index-by-index deletion on a densely populated array.
arr["0"] = "zero";
arr["1"] = "one";
arr["2"] = "two";
arr["3"] = "three";
arr["4"] = "four";
arr["6"] = "six";
arr["7"] = "seven";
arr["8"] = "eight";
arr["9"] = "nine";

// There is deliberately no canonical "5" property.
arr["-0"] = "negative_zero";
arr["05"] = "leading_zero";
arr[" 5"] = "leading_space";
arr["+5"] = "leading_plus";
arr["-5"] = "negative_five";
arr["5 "] = "trailing_space";

trace("=== Initial state ===");
trace("Length: " + arr.length);
trace("0: " + arr.hasOwnProperty("0"));
trace("4: " + arr.hasOwnProperty("4"));
trace("5: " + arr.hasOwnProperty("5"));
trace("6: " + arr.hasOwnProperty("6"));
trace("9: " + arr.hasOwnProperty("9"));
trace("-0: " + arr.hasOwnProperty("-0"));
trace("05: " + arr.hasOwnProperty("05"));
trace(" 5: " + arr.hasOwnProperty(" 5"));
trace("+5: " + arr.hasOwnProperty("+5"));
trace("-5: " + arr.hasOwnProperty("-5"));
trace("5 : " + arr.hasOwnProperty("5 "));

trace("");
trace("=== Set length to 5 without canonical index 5 ===");

arr.length = 5;

trace("Length: " + arr.length);
trace("4: " + arr.hasOwnProperty("4"));
trace("5: " + arr.hasOwnProperty("5"));
trace("6: " + arr.hasOwnProperty("6"));
trace("9: " + arr.hasOwnProperty("9"));
trace("-0: " + arr.hasOwnProperty("-0"));
trace("05: " + arr.hasOwnProperty("05"));
trace(" 5: " + arr.hasOwnProperty(" 5"));
trace("+5: " + arr.hasOwnProperty("+5"));
trace("-5: " + arr.hasOwnProperty("-5"));
trace("5 : " + arr.hasOwnProperty("5 "));

trace("");
trace("=== Set length to 0 ===");

arr.length = 0;

trace("Length: " + arr.length);
trace("0: " + arr.hasOwnProperty("0"));
trace("4: " + arr.hasOwnProperty("4"));
trace("-0: " + arr.hasOwnProperty("-0"));
trace("05: " + arr.hasOwnProperty("05"));
trace(" 5: " + arr.hasOwnProperty(" 5"));
trace("+5: " + arr.hasOwnProperty("+5"));
trace("-5: " + arr.hasOwnProperty("-5"));
trace("5 : " + arr.hasOwnProperty("5 "));
