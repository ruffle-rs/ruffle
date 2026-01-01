function printTextFormat(tf) {
    trace("  size = " + tf.size + ";");
    trace("  align = " + tf.align + ";");
    trace("  blockIndent = " + tf.blockIndent + ";");
    trace("  bold = " + tf.bold + ";");
    trace("  bullet = " + tf.bullet + ";");
    trace("  color = " + tf.color + ";");
    trace("  font = " + tf.font + ";");
    trace("  indent = " + tf.indent + ";");
    trace("  italic = " + tf.italic + ";");
    trace("  leading = " + tf.leading + ";");
    trace("  leftMargin = " + tf.leftMargin + ";");
    trace("  letterSpacing = " + tf.letterSpacing + ";");
    trace("  rightMargin = " + tf.rightMargin + ";");
    trace("  size = " + tf.size + ";");
    trace("  tabStops = " + tf.tabStops + ";");
    trace("  target = " + tf.target + ";");
    trace("  underline = " + tf.underline + ";");
    trace("  url = " + tf.url + ";");
}

trace("text.getNewTextFormat()");
printTextFormat(text.getNewTextFormat());
trace("text.getTextFormat()");
printTextFormat(text.getTextFormat());

text.text = "x";

trace("text.getTextFormat()");
printTextFormat(text.getTextFormat());

trace("text.getTextFormat(0,0)");
printTextFormat(text.getTextFormat(0,0));

text.text = "";

trace("text.getTextFormat()");
printTextFormat(text.getTextFormat());
