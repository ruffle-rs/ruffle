// Test that getTextExtent handles undefined width correctly.
// When width is undefined, it should be treated as unconstrained
// (same as omitting the argument).

var fmt = new TextFormat();
fmt.size = 12;

// getTextExtent with no width (single arg) - should auto-size
var metrics1 = fmt.getTextExtent("Hello World");
trace("No width - textFieldWidth > 0: " + (metrics1.textFieldWidth > 0));
trace("No width - textFieldHeight > 0: " + (metrics1.textFieldHeight > 0));

// getTextExtent with undefined width - should behave same as no width
var metrics2 = fmt.getTextExtent("Hello World", undefined);
trace("Undefined width - textFieldWidth > 0: " + (metrics2.textFieldWidth > 0));
trace("Undefined width - textFieldHeight > 0: " + (metrics2.textFieldHeight > 0));

// The widths should be the same (both unconstrained)
trace("Same textFieldWidth: " + (metrics1.textFieldWidth == metrics2.textFieldWidth));
trace("Same textFieldHeight: " + (metrics1.textFieldHeight == metrics2.textFieldHeight));

// getTextExtent with a valid width constraint
var metrics3 = fmt.getTextExtent("Hello World", 50);
trace("Width 50 - textFieldWidth > 0: " + (metrics3.textFieldWidth > 0));
trace("Width 50 - textFieldHeight > 0: " + (metrics3.textFieldHeight > 0));

// Test that a real object's valueOf IS called (coercion happens)
var noisyFloat = new Object();
noisyFloat.valueOf = function() {
    trace("coerced");
    return 42.0;
};
var metrics4 = fmt.getTextExtent("Hello World", noisyFloat);
trace("Object width - textFieldWidth > 0: " + (metrics4.textFieldWidth > 0));
