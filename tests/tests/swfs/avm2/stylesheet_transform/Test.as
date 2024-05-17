package  {
	import flash.display.MovieClip;
	import flash.text.StyleSheet;
	import flash.text.TextFormat;
	
	
	public class Test extends MovieClip {
		// this is only relevant properties, some can't be set through css
		var textFormatProperties = [
			"align",
			"bold",
			"color",
			"font",
			"indent",
			"italic",
			"kerning",
			"leading",
			"leftMargin",
			"letterSpacing",
			"rightMargin",
			"size",
			"underline",
			"display" // undocumented!
		];

		var interestingNumbers = [
			"",
			"50",
			"50.5 px",
			"50 pt",
			"50xx",
			" 50",
			"x50",
			"0",
			"-50",
			"09"
		];

		public function Test() {
			var styleSheet: StyleSheet = new StyleSheet();

			test("Empty", styleSheet, {});
			test("Null", styleSheet, null);
			test("Undefined", styleSheet, undefined);
			test("Number", styleSheet, 5);

			testSingleProperty("Color", styleSheet, "color", "color", [
				"red",
				"",
				"123",
				"#",
				"#1",
				"#12",
				"#123",
				"#1234",
				"#12345",
				"#123456",
				"#1234567",
				"#red"
			]);

			testSingleProperty("Display", styleSheet, "display", "display", [
				"inline",
				"block",
				"none",
				"invalid",
				""
			]);

			testSingleProperty("Font Family", styleSheet, "fontFamily", "font", [
				"",
				"mono, sans-serif, serif",
				"a b c, d e   f  ,  ,  g h",
				"Times New Roman"
			]);

			testSingleProperty("Font Size", styleSheet, "fontSize", "size", interestingNumbers);

			testSingleProperty("Font Style", styleSheet, "fontStyle", "italic", [
				"",
				"bold",
				"italic",
				"normal"
			]);

			testSingleProperty("Font Weight", styleSheet, "fontWeight", "bold", [
				"",
				"bold",
				"italic",
				"normal"
			]);

			testSingleProperty("Kerning", styleSheet, "kerning", "kerning", [
				"",
				"true",
				"false",
				"lots",
				"50",
				"0"
			]);

			testSingleProperty("Leading", styleSheet, "leading", "leading", interestingNumbers);

			testSingleProperty("Letter Spacing", styleSheet, "letterSpacing", "letterSpacing", interestingNumbers);

			testSingleProperty("Margin Left", styleSheet, "marginLeft", "leftMargin", interestingNumbers);

			testSingleProperty("Margin Right", styleSheet, "marginRight", "rightMargin", interestingNumbers);

			testSingleProperty("Text Align", styleSheet, "textAlign", "align", [
				"",
				"invalid",
				"left",
				"right",
				"justify",
				"center"
			]);

			testSingleProperty("Text Decoration", styleSheet, "textDecoration", "underline", [
				"",
				"bold",
				"none",
				"underline"
			]);

			testSingleProperty("Text Indent", styleSheet, "textIndent", "indent", interestingNumbers);

			test("Every property is valid", styleSheet, {
				"color": "#FF0000",
				"display": "block",
				"fontFamily": "sans-serif",
				"fontSize": "75px",
				"fontStyle": "italic",
				"fontWeight": "bold",
				"kerning": "true",
				"leading": "5",
				"letterSpacing": "0",
				"marginLeft": "0",
				"marginRight": "0",
				"textAlign": "justify",
				"textDecoration": "underline",
				"textIndent": "0"
			});
		}

		function test(name: String, styleSheet: StyleSheet, style: *) {
			trace("/// " + name);
			dumpStyle("style", style);
			trace("");
			try {
				dumpFormat("format", styleSheet.transform(style));
			} catch (e) {
				// [NA] The exact error message deviates at time of writing because of int vs Number
				trace("! " + e.errorID);
			}
			trace("");
		}

		function testSingleProperty(name: String, styleSheet: StyleSheet, property: String, transformProperty: String, values: Array) {
			trace("/// " + name);
			
			for each (var value in values) {
				trace("// styleSheet.transform({" + escapeString(property) + ": " + escapeString(value) + "})");
				try {
					var input = {};
					input[property] = value;
					var result = styleSheet.transform(input);
					dumpValue(transformProperty, result[transformProperty]);
				} catch (e) {
					trace("! " + e);
				}
			}

			trace("");
		}

		function dumpStyle(name: String, style: *) {
			if (style === undefined) {
				trace( name + " = undefined");
			} else if (style === null) {
				trace( name + " = null");
			} else {
				var first = true;

				// Sort is not deterministic
				var sortedKeys = [];
				for (var key in style) {
					sortedKeys.push(key);
				}
				sortedKeys.sort();

				for each (var key in sortedKeys) {
					if (first) {
						first = false;
						trace(name + " = {");
					}
					dumpValue("   " + escapeString(key), style[key]);
				}

				if (first) {
					trace(name + " = {}");
				} else {
					trace("}");
				}
			}
		}

		function dumpFormat(name: String, format: *) {
			if (format === undefined) {
				trace( name + " = undefined");
			} else if (format === null) {
				trace( name + " = null");
			} else {
				var first = true;

				for each (var key in textFormatProperties) {
					if (first) {
						first = false;
						trace(name + " = {");
					}
					dumpValue("   " + escapeString(key), format[key]);
				}

				if (first) {
					trace(name + " = {}");
				} else {
					trace("}");
				}
			}
		}

		function dumpValue(name: String, value: *) {
			if (value === undefined) {
				return;
			} else if (value === null) {
				trace(name + " = null");
			} else if (value is Number) {
				trace(name + " = number " + value);
			} else if (value is String) {
				trace(name + " = string " + escapeString(value));
			} else if (value is Boolean) {
				trace(name + " = boolean " + value);
			} else if (value is Object) {
				trace(name + " = object " + value);
			} else {
				trace(name + " = unknown " + value);
			}
		}

		function escapeString(input: String): String {
			var output:String = "\"";
			for (var i:int = 0; i < input.length; i++) {
				var char:String = input.charAt(i);
				switch (char) {
					case "\\":
						output += "\\\\";
						break;
					case "\"":
						output += "\\\"";
						break;
					case "\n":
						output += "\\n";
						break;
					case "\r":
						output += "\\r";
						break;
					case "\t":
						output += "\\t";
						break;
					default:
						output += char;
				}
			}
			return output + "\"";
		}
	}
}
