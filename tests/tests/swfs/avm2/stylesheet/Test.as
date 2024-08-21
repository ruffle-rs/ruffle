package  {
	import flash.display.MovieClip;
	import flash.text.StyleSheet;
	
	
	public class Test extends MovieClip {
		public function Test() {
			var styleSheet: StyleSheet = new StyleSheet();

			test("New stylesheet", styleSheet, function() {});

			test("setStyle", styleSheet, function() {
				var object = {color: "red", invalid: 32, kerning: "blue", nested: {key: "value"}};
				styleSheet.setStyle("blank", {});
				styleSheet.setStyle("null", null);
				styleSheet.setStyle("undefined", undefined);
				styleSheet.setStyle("string", "test { color: red; }");
				styleSheet.setStyle("OBJECT", {color: "blue"});
				styleSheet.setStyle("Object", object);

				if (styleSheet.getStyle("object") === object) {
					trace("FAIL: object should be shallow cloned, not used directly");
				}
				if (styleSheet.getStyle("object").nested !== object.nested) {
					trace("FAIL: object values should be used directly, not cloned");
				}
				dumpStyle("styleSheet.getStyle(\"OBJect\")", styleSheet.getStyle("OBJect"));
			});

			test("setStyle again", styleSheet, function() {
				styleSheet.setStyle("object", {kerning: 5});
			});

			test("clear", styleSheet, function() {
				styleSheet.clear();
			});

			test("parseCSS", styleSheet, function() {
				styleSheet.parseCSS("one { color: red; }");
			});

			test("parseCSS: malformed", styleSheet, function() {
				styleSheet.parseCSS("one { color: blue; }; this isn't valid!");
			});

			test("parseCSS: replacing vs merging", styleSheet, function() {
				styleSheet.parseCSS("one { kerning: 5 }");
				styleSheet.parseCSS("two { color: red; }");
			});

			test("parseCSS: two selectors, one definition", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a, b { kerning: 5 }");
			});

			test("parseCSS: selector with spaces", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("long key { kerning: 5 }");
			});

			test("parseCSS: selector with special characters", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a.b.c-d!#@$😜 { worked: honestly, I'm surprised too! }");
			});

			test("parseCSS: case sensitivity in selector", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("" +
					"key { color: red; }" +
					"KEY { color: blue; }" +
					"KeY { color: pink; }" +
					"");
			});

			test("parseCSS: no selector", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("{color:}");
			});

			test("parseCSS: escapes?", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("test\\ {color: \\}");
			});

			test("parseCSS: strings?", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("\"test\" {color: \"red\"}");
			});

			test("parseCSS: selector names contains }", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a}{}");
			});

			test("parseCSS: unclosed block", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{");
			});

			test("parseCSS: only selector", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("name");
			});

			test("parseCSS: empty property name", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{:}");
			});

			test("parseCSS: no colon and unclosed", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{name");
			});

			test("parseCSS: no colon", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{name}");
			});

			test("parseCSS: many semicolons", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{;;key: value;other: value;;}");
				styleSheet.parseCSS("b{;;key: value;;other: value;}");
				styleSheet.parseCSS("c{;;key: value;other: value;}");
				styleSheet.parseCSS("d{key: value;;}");
				styleSheet.parseCSS("e{key;other: value;}");
				styleSheet.parseCSS("f{;:value}");
				styleSheet.parseCSS("g{key:value; }");
			});

			test("parseCSS: property name is actually a whole other block", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{name}b{:value}");
			});

			test("parseCSS: empty property name and unclosed", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{:");
			});

			test("parseCSS: unclosed block after name given", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{key:");
			});

			test("parseCSS: closed block after name given", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{key:}");
			});

			test("parseCSS: unclosed block after value given", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{key:value");
			});

			test("parseCSS: unclosed block after semicolon", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{key:;");
			});

			test("parseCSS: unclosed empty name and value", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{:;");
			});

			test("parseCSS: case sensitivity in properties", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("key {" +
					"property: value; " +
					"Property: Value; " +
					"PROPERTY: VALUE; " +
					"}");
			});

			test("parseCSS: whitespace in property name", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("key {" +
					"property with space: value; " +
					"}");
			});

			test("parseCSS: whitespace in property value", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("key {" +
					"  property  :   test    value    " +
					"}");
			});

			test("parseCSS: name transformation", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("key {" +
					"name-with-dashes: value" +
					"  name-with-dashes-and-spaces  : value;" +
					"this--one--has--two--dashes: value;" +
					"UPPER-DASHES: value;" +
					"}");
			});

			test("parseCSS: last property without semicolon trimmed correctly", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("a{key1: value; key2: value \r\n} b{key3: value \n}");
			});
			
			test("parseCSS: comment", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("/* comment */ a{key: value;}");
			});

			test("parseCSS: unclosed comment", styleSheet, function() {
				styleSheet.clear();
				styleSheet.parseCSS("/* comment a{key: value;}");
			});

			styleSheet = new AlwaysRedStyleSheet();
			test("transform", styleSheet, function() {
				styleSheet.setStyle("anything", "blue");
			});
		}

		function test(name: String, styleSheet: StyleSheet, fn: Function) {
			trace("/// " + name);
			try {
				fn();
			} catch (err) {
				trace("! " + err);
			}
			dumpStyleSheet(styleSheet);
			trace("");
		}

		function dumpStyleSheet(styleSheet: StyleSheet) {
			// Sort is not deterministic
			var sortedNames = [];
			for each (var name in styleSheet.styleNames) {
				sortedNames.push(name);
			}
			sortedNames.sort();


			for each (var name in sortedNames) {
				dumpStyle("styleSheet.getStyle(" + escapeString(name) + ")", styleSheet.getStyle(name));
			}

			// Docs say it should always return null.
			// Docs are liars.
			dumpStyle("styleSheet.getStyle(\"nonexistant\")", styleSheet.getStyle("nonexistant"));
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
					dumpValue(escapeString(key), style[key]);
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
				trace("    " + name + " = null");
			} else if (value is Number) {
				trace("    " + name + " = number " + value);
			} else if (value is String) {
				trace("    " + name + " = string " + escapeString(value));
			} else if (value is Object) {
				trace("    " + name + " = object " + value);
			} else {
				trace("    " + name + " = unknown " + value);
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
