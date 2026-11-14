var arr:Array = new Array();

arr.length = 1000000;

// Canonical indices.
arr["0"] = "zero";
arr["5"] = "five";
arr["499999"] = "below_boundary";
arr["500000"] = "at_boundary";
arr["999999"] = "last";

// Non-canonical property names.
arr["-0"] = "negative_zero";
arr["05"] = "leading_zero";
arr[" 5"] = "leading_space";
arr["+5"] = "leading_plus";
arr["-5"] = "negative_five";
arr["5 "] = "trailing_space";

trace("=== Initial state ===");
trace("Length: " + arr.length);

trace("");
trace("=== Set length to 500000 ===");

arr.length = 500000;

trace("Length: " + arr.length);
trace("0: " + arr.hasOwnProperty("0"));
trace("5: " + arr.hasOwnProperty("5"));
trace("499999: " + arr.hasOwnProperty("499999"));
trace("500000: " + arr.hasOwnProperty("500000"));
trace("999999: " + arr.hasOwnProperty("999999"));

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
trace("5: " + arr.hasOwnProperty("5"));
trace("499999: " + arr.hasOwnProperty("499999"));

trace("-0: " + arr.hasOwnProperty("-0"));
trace("05: " + arr.hasOwnProperty("05"));
trace(" 5: " + arr.hasOwnProperty(" 5"));
trace("+5: " + arr.hasOwnProperty("+5"));
trace("-5: " + arr.hasOwnProperty("-5"));
trace("5 : " + arr.hasOwnProperty("5 "));

// Repeat the boundary case without ever creating the canonical "5" key.
var missingFive:Array = new Array();
missingFive.length = 1000000;
missingFive["0"] = "zero";
missingFive["999999"] = "last";
missingFive["-0"] = "negative_zero";
missingFive["05"] = "leading_zero";
missingFive[" 5"] = "leading_space";
missingFive["+5"] = "leading_plus";
missingFive["-5"] = "negative_five";
missingFive["5 "] = "trailing_space";

trace("");
trace("=== Sparse array without canonical index 5 ===");
trace("Length: " + missingFive.length);
trace("5: " + missingFive.hasOwnProperty("5"));
trace("05: " + missingFive.hasOwnProperty("05"));

missingFive.length = 5;

trace("Length after truncation: " + missingFive.length);
trace("0: " + missingFive.hasOwnProperty("0"));
trace("5: " + missingFive.hasOwnProperty("5"));
trace("999999: " + missingFive.hasOwnProperty("999999"));
trace("-0: " + missingFive.hasOwnProperty("-0"));
trace("05: " + missingFive.hasOwnProperty("05"));
trace(" 5: " + missingFive.hasOwnProperty(" 5"));
trace("+5: " + missingFive.hasOwnProperty("+5"));
trace("-5: " + missingFive.hasOwnProperty("-5"));
trace("5 : " + missingFive.hasOwnProperty("5 "));
