package {
	import flash.utils.getQualifiedClassName;
	
	
	public class Test {
		public static function run() {
			var outer = <outer firstAttr="Hello" sharedAttr="Outer level">
				<firstChild sharedAttr="First child" uniqueAttr="My unique string">
					<nested sharedAttr="Nested elemeent"/>
				</firstChild>
				<child sharedAttr="Second child" secondChildAttr="Second child attr"/>
				<child sharedAttr="Third child" thirdChildAttr="Third child attr"/>
			</outer>;
			
			var customKey = {
				toString: function() {
					return "firstAttr";
				}
			};
			
			trace("outer.attribute(new QName(null, \"firstAttr\")) = " + outer.attribute(new QName(null, "firstAttr")));
			trace("outer.attribute(\"firstAttr\") = " + outer.attribute("firstAttr"));
			trace("outer.attribute(customKey) = " + outer.attribute(customKey));
			trace("outer.attribute(\"sharedAttr\") = " + outer.attribute("sharedAttr"));
			trace("outer.attribute(\"missingAttr\") = " + outer.attribute("missingAttr"));
			trace("Types: " + getQualifiedClassName(outer.attribute("firstAttr")) + " " + getQualifiedClassName(outer.attribute("sharedAttr")) + " " + getQualifiedClassName(outer.attribute("missingAttr")));
			
			trace("outer.attributes() = " + outer.attributes());
			trace("outer.attributes().length() = " + outer.attributes().length());
			
			trace("outer.child.attributes() = " + outer.child.attributes());
			trace("outer.child.attributes().length() = " + outer.child.attributes().length());
			trace("new XMLList().attributes().length() = " + new XMLList().attributes().length());
		
			var otherCustomKey = {
				toString: function() {
					return "sharedAttr";
				}
			}
			
			trace("outer.child.attribute(otherCustomKey) = " + outer.child.attribute(otherCustomKey));
			trace("outer.child.attribute(new QName(null, \"sharedAttr\")) = " + outer.child.attribute(new QName(null, "sharedAttr")));
			trace("outer.child.attribute(\"sharedAttr\") = " + outer.child.attribute("sharedAttr"));
			trace("outer.child.attribute(\"sharedAttr\").length() = " + outer.child.attribute("sharedAttr").length());
			trace("outer.child.attribute(\"missingAttr\").length() = " + outer.child.attribute("missingAttr").length());
		
			outer.@newAttr = "Some value";
			trace("outer.@newAttr = " + outer.@newAttr);
			trace("outer.attribute('newAttr') = " + outer.attribute('newAttr'));
		
			outer.@secondNewAttr = 5;
			trace("outer.@secondNewAttr = " + outer.@secondNewAttr);
			trace("outer.attribute('secondNewAttr') = " + outer.attribute('secondNewAttr'));
		
			outer.@thirdNewAttr = otherCustomKey;
			trace("outer.@thirdNewAttr = " + outer.@thirdNewAttr);
			trace("outer.attribute('thirdNewAttr') = " + outer.attribute('thirdNewAttr'));
		
			outer.@name = "Custom attr value";
			trace("outer.@name = " + outer.@name);
			trace("outer.attribute('name') = " + outer.attribute('name'));
			trace("outer.name() = " + outer.name());

			for each (var attr in outer.attributes()) {
				trace("Attr: " + attr.name() + " = " + attr);
			}
		}
	}
}
