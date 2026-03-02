// Test that getTextExtent handles undefined/NaN width correctly.
// When width is undefined or NaN, it should be treated as unconstrained
// (same as omitting the argument), not as width=NaN which produces
// textFieldWidth=0.

var fmt = new TextFormat();
fmt.size = 12;
fmt.font = "_sans";

// getTextExtent with no width (single arg) - should auto-size
var metrics1 = fmt.getTextExtent("Hello World");
trace("No width - textFieldWidth > 0: " + (metrics1.textFieldWidth > 0));
trace("No width - textFieldHeight > 0: " + (metrics1.textFieldHeight > 0));

// getTextExtent with undefined width - should behave same as no width
var metrics2 = fmt.getTextExtent("Hello World", undefined);
trace("Undefined width - textFieldWidth > 0: " + (metrics2.textFieldWidth > 0));
trace("Undefined width - textFieldHeight > 0: " + (metrics2.textFieldHeight > 0));

// The widths should be the same (both unconstrained)
trace("Same width: " + (metrics1.textFieldWidth == metrics2.textFieldWidth));

// getTextExtent with a valid width constraint
var metrics3 = fmt.getTextExtent("Hello World", 50);
trace("Width 50 - textFieldWidth > 0: " + (metrics3.textFieldWidth > 0));
trace("Width 50 - textFieldHeight > 0: " + (metrics3.textFieldHeight > 0));

// Verify setTextFormat with a format derived from getTextExtent doesn't crash
this.createTextField("test_field", 1, 0, 0, 200, 100);
test_field.text = "Hello World";
test_field.setTextFormat(fmt);
trace("setTextFormat after getTextExtent: OK");
trace("Text preserved: " + test_field.text);
