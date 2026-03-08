// Test that getTextExtent handles undefined width correctly.
// When width is undefined, it should be treated as unconstrained
// (same as omitting the argument).

// Helper to trace all metrics from getTextExtent
function traceMetrics(label, m)
{
    trace(label + ":");
    trace("  textFieldWidth = " + m.textFieldWidth);
    trace("  textFieldHeight = " + m.textFieldHeight);
    trace("  width = " + m.width);
    trace("  height = " + m.height);
    trace("  ascent = " + m.ascent);
    trace("  descent = " + m.descent);
}

// === Embedded font tests ===
trace("=== Embedded font ===");

var fmt = new TextFormat();
fmt.size = 20;
fmt.font = "TestFont";

// getTextExtent with no width (single arg) - should auto-size
var metrics1 = fmt.getTextExtent("abcd");
traceMetrics("No width", metrics1);

// getTextExtent with undefined width - should behave same as no width
var metrics2 = fmt.getTextExtent("abcd", undefined);
traceMetrics("Undefined width", metrics2);

// The widths should be the same (both unconstrained)
trace("Same textFieldWidth: " + (metrics1.textFieldWidth == metrics2.textFieldWidth));
trace("Same width: " + (metrics1.width == metrics2.width));

// getTextExtent with a valid width constraint
var metrics3 = fmt.getTextExtent("abcd", 50);
traceMetrics("Width 50", metrics3);

// === Non-existent embedded font ===
trace("=== Non-existent font ===");

var fmt2 = new TextFormat();
fmt2.size = 20;
fmt2.font = "DoesNotExist";

var metrics4 = fmt2.getTextExtent("abcd");
traceMetrics("Non-existent font", metrics4);

var metrics5 = fmt2.getTextExtent("abcd", undefined);
traceMetrics("Non-existent font undefined width", metrics5);

trace("Non-existent same textFieldWidth: " + (metrics4.textFieldWidth == metrics5.textFieldWidth));

// === Side-effect test: valueOf IS called for real objects ===
trace("=== Side-effect test ===");

var noisyFloat = new Object();
noisyFloat.valueOf = function()
{
    trace("coerced");
    return 42.0;
};
var metrics6 = fmt.getTextExtent("abcd", noisyFloat);
trace("Object width - textFieldWidth > 0: " + (metrics6.textFieldWidth > 0));
