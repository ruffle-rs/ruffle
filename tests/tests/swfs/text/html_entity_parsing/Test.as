package {

import flash.display.Sprite;
import flash.text.TextField;

public class Test extends Sprite {
	public function Test() {
		var testField:TextField = new TextField;
		var tests:Array = new Array(
			// all the explicit entities supported
			"This is an ampersand: &amp;",
			"This is a less-than sign: &lt;",
			"This is a greater-than sign: &gt;",
			"This is a quotation mark: &quot;",
			"This is an apostrophe: &apos;",
			"This is a non-breaking space: &nbsp;",

			// an entity right at the beginning
			"&amp;",
			"&quot; is a quotation mark",

			// many entities
			"Suppose we have two objects, &quot;foo&quot; and &quot;bar&quot;",

			// adjacent entities
			"This is how you write an empty string: &quot;&quot;",

			// an entity-alike that should not resolve
			"This entity should not be valid: &jjjjj;",

			// no numeral indicator
			"This entity should not be valid: &1;",
			"This entity should not be valid: &x2;",

			// an empty entity
			"This entity should not be valid: &;",

			// no entity start
			"This is not an entity: amp;",
			";",

			// no entity end
			"This is not an entity: &amp",
			"&",

			// whitespace inside explicit entities
			"This is not an entity: &amp ;",
			"This is not an entity: & amp;",
			"This is not an entity: & amp ;",

			// basic numeric tests
			"This is a greater-than sign: &#62;",
			"This is an ampersand: &#x26;",

			// no numerals
			"This is not an entity: &#;",
			"This is not an entity: &#x;",

			// negative numbers are parsed as u16s
			"This is a halfwidth Hangul wae: &#-50;",
			"This is a unified Chinese yào: &#x-8000;",

			// make sure we support positive numbers too!
			"This is a left parenthesis: &#+40;",
			"This is a comma: &#x+2c;",
			
			// mind the gap!
			"This is a less-than sign: &# 60;",
			"This is a question mark: &#x  3f;",
			"This is a greater-than sign: &#62   ;",
			"This is an ampersand: &#x26     ;",
			"This is a right parenthesis: &# 41  ;",
			"This is an asterisk: &#x  2a ;",

			// newlines too!
			"This is a greater-than sign: &#\n62;",
			"This is an ampersand: &#x\n26;",
			"This is a colon: &#58\n;",
			"This is a semicolon: &#x3b\n;",

			// but only around the numbers!
			"This is not an entity: & #58;",
			"This is not an entity: &#  x3b;",
			"This is not an entity: &#+ 40;",
			"This is not an entity: &#x- 2c;",
			"This is a colon: &#58 0;",
			"This is a semicolon: &#x3b 0a;",

			// invalid numerals are truncated
			"This is truncated to a left parenthesis: &#40a;",
			"This is truncated to a less-than sign: &#60-9312;",
			"This is truncated to a comma: &#x2cm;",
			"This is truncated to a question mark: &#x3FQ3a01;",

			// no arithmetic
			"This is not an entity: &#(20+6);",

			// even though HTML allows capital X for hex, Flash Player rejects it like with XML
			"Flash rejects this entity even though it is valid in HTML: &#X28;",

			// both lowercase and uppercase hex digits are allowed...
			"This is a colon: &#x3a;",
			"This is a semicolon: &#x3B;",

			// ... including together
			"This is a registered trademark symbol: &#xaE;",
			"This is a not sign: &#xAc;",

			// leading zeroes are allowed and do not change the parsing:
			"This is a right parenthesis: &#041;",
			"This is an asterisk: &#x02a;",

			// short numbers
			"This is a horizontal tab: &#9;",
			"This is a horizontal tab: &#x9;", // the only well-behaved single-hexit character

			// long numbers
			"This is a unified Chinese nǐ: &#20320;",
			"This is a unified Chinese kǎo: &#x8003;",

			// codepoints which don't fit in 16 bits are truncated
			"This should be a playing card black joker, but Flash truncates it to fit in a u16: &#127183;",
			"This should be a Mahjong tile red dragon, but Flash truncates it to fit in a u16: &#x1F004;",

			// obnoxiously long numbers
			"This is a Hangul tyaelp: &#1311768467463786786;",
			"This is a Hangul tyaelh: &#x123456789ABCD123;",

			// a double-decode trap
			"This is a double-encoded quotation mark and, unless first XML-parsed, should only be singly decoded: &amp;#x22;"
		);
		for (var s:String in tests) {
			trace("\nSource: " + tests[s]);
			testField.htmlText = tests[s];
			trace("Parsed: " + testField.text);
		}
	}
}

}
