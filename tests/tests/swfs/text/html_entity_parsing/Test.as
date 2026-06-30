package {
	import flash.display.Sprite;
	public class Test extends Sprite {}
}

import flash.text.TextField;
var testField:TextField = new TextField;
testField.htmlText =
	// all the explicit entities supported...
	"This is an ampersand: &amp;\n" +
	"This is a less-than sign: &lt;\n" +
	"This is a greater-than sign: &gt;\n" +
	"This is a quotation mark: &quot;\n" +
	"This is an apostrophe: &apos;\n" +
	"This is a non-breaking space: &nbsp;\n" +
	// numeric tests...
	"This is a decimal-encoded greater-than sign: &#62;\n" +
	"This is a hex-encoded ampersand: &#x26;\n" +
	// and a double-decode trap
	"This is a double-encoded quotation mark and, unless first XML-parsed, should only be singly decoded: &amp;#x22;";
trace(testField.text);
