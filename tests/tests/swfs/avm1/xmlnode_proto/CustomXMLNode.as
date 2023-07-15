class CustomXMLNode extends XMLNode {
	static function test() {
		XMLNode.prototype = new CustomXMLNode();
		var xml = new XML("<a><b>test</b></a>");
		xml.firstChild.checkForSuccess();
	}
	
	function checkForSuccess() {
		trace("Success!");
	}
}