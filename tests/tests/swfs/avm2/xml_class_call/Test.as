package {
	public class Test {
		public function Test() {
			var xml = XML("Some string");
			trace("From string:");
			trace(xml.children().length());
			trace(xml.toString());
			
			var complexXML = XML("<outer><inner><child>Hello</child></inner></outer>");
			trace("From complex string:")
			trace(complexXML.children().length());
			trace(complexXML.name());
			
			trace("XMLList from string:");
			trace(XMLList("<p>First element</p><p>Second element</p>"));
		}
	}
}