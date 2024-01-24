function traceTextFormat(tf) {
    trace("    tf.bold:");
    trace("      " + tf.bold);
    trace("    tf.italic:");
    trace("      " + tf.italic);
    trace("    tf.underline:");
    trace("      " + tf.underline);
    trace("    tf.align");
    trace("      " + tf.align);
    trace("    tf.blockIndent");
    trace("      " + tf.blockIndent);
    trace("    tf.bullet");
    trace("      " + tf.bullet);
    trace("    tf.color");
    trace("      " + tf.color);
    trace("    tf.font");
    trace("      " + tf.font);
    trace("    tf.indent");
    trace("      " + tf.indent);
    trace("    tf.kerning");
    trace("      " + tf.kerning);
    trace("    tf.leading");
    trace("      " + tf.leading);
    trace("    tf.leftMargin");
    trace("      " + tf.leftMargin);
    trace("    tf.letterSpacing");
    trace("      " + tf.letterSpacing);
    trace("    tf.rightMargin");
    trace("      " + tf.rightMargin);
    trace("    tf.size");
    trace("      " + tf.size);
    trace("    tf.tabStops.join(\',\')");
    trace("      " + tf.tabStops.join(","));
    trace("    tf.target");
    trace("      " + tf.target);
    trace("    tf.url");
    trace("      " + tf.url);
}

function testField(text) {
    trace("  ==== text");
    trace("    " + text.text);
    trace("  ==== html");
    trace("    " + text.htmlText);
    trace("  ==== default text format");
    traceTextFormat(text.getTextFormat());
}

trace("==== regular");
testField(textRegular);
trace("==== bold");
testField(textBold);
trace("==== italic");
testField(textItalic);
trace("==== bold italic");
testField(textBoldItalic);

trace("==== regular html");
testField(textRegularHtml);
trace("==== bold html");
testField(textBoldHtml);
trace("==== italic html");
testField(textItalicHtml);
trace("==== bold italic html");
testField(textBoldItalicHtml);
