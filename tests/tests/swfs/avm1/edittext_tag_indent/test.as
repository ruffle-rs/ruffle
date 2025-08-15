Stage.scaleMode = "noScale";

function traceIndent(tf) {
    trace("Indent for " + tf + ":");
    trace(tf.getNewTextFormat().indent);
    trace(tf.getTextFormat(0, 1).indent);
}

function testSettingIndent(value) {
    var tf = new TextFormat();
    tf.indent = value;
    text1.setNewTextFormat(tf);
    trace("Setting " + value + " -> " + text1.getNewTextFormat().indent);
}

traceIndent(text1);
traceIndent(text2);
traceIndent(text3);
traceIndent(text4);
traceIndent(text5);

testSettingIndent(0.0);
testSettingIndent(0.1);
testSettingIndent(0.4);
testSettingIndent(0.5);
testSettingIndent(0.6);
testSettingIndent(0.8);
testSettingIndent(1.0);
testSettingIndent(1.1);
testSettingIndent(1.4);
testSettingIndent(1.5);
testSettingIndent(1.6);
testSettingIndent(1.8);
testSettingIndent(2.0);
testSettingIndent(-10);
testSettingIndent(100);
testSettingIndent(64010);
