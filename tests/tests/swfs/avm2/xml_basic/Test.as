package {
	public class Test {
		public static function run() {
			var soapXML:XML = 
					<soap:Envelope xmlns:soap="http://www.w3.org/2001/12/soap-envelope"
						soap:encodingStyle="http://www.w3.org/2001/12/soap-encoding">

									<soap:Body xmlns:wx = "http://example.com/weather">
							<wx:forecast>
								<wx:city>Quito</wx:city>
							</wx:forecast>
						</soap:Body>
					</soap:Envelope>;
					
			trace(soapXML.localName()); // Envelope
			trace(XML.prototype.localName.call(soapXML));
			
			var simpleXML:XML = <outerElem><innerElem><p>Hello world</p></innerElem></outerElem>;
			trace("simpleXML.innerElem.p = " + simpleXML.innerElem.p);
			
			trace("XML.prototype.toString() = " + XML.prototype.toString());
			
			var noArgs = new XML();
			trace("noArgs.toString() = " + noArgs.toString());
			trace("XML.prototype.toString.call(noArgs): " + XML.prototype.toString.call(noArgs));
			trace("noArgs.toXMLString() = " + noArgs.toXMLString());
			
			var nullArg = new XML(null);
			trace("nullArg.toString() = " + nullArg.toString());
			trace("nullArg.toString() = " + nullArg.toXMLString());
			
			var undefinedArg = new XML(undefined);
			trace("undefinedArg.toString() = " + undefinedArg.toString());
			trace("undefinedArg.toXMLString() = " + undefinedArg.toString());
			
			var plainString:XML = new XML("Hello");
			trace("plainString.toString() = " + plainString.toString());
			trace("plainString.toXMLString() = " + plainString.toString());
			
			var list = new XMLList("<p>First</p><p>Second</p>");
			trace("List children: " + list.length());
			
			trace("List first child: " + list[0]);
			trace("List second child: " + list[1]);
			
			var a = <a><x>asdf</x></a>;
			var a1 = a.x;
			var a2 = a1[0];
			var b1 = a.x;
			var b2 = b1[0];
			trace("XMLList strict equal: " + (a1 === b1));
			trace("XML strict equal: " + (a2 === b2));
			
			var weird = <outer><name>My Name</name></outer>;
			trace("Get 'name' property': " + weird.name);
			trace("Get 'AS#::name' property': " + (typeof a.AS3::name));
			
			var cdata = <![CDATA[My
Multiline
CDATA
]]>;
			
			trace(cdata);
			trace(cdata.toXMLString());
			
var declaration_doctype = new XML("<?xml version = \"1.0\" encoding = \"UTF-8\" standalone = \"yes\" ?> <!DOCTYPE person [<!ELEMENT name (#PCDATA)> ]> <p>Skipped everything else</p>");
trace(declaration_doctype.toString());

			var commentsAndPI = <wrapper><?display table-view?> <!-- Some comment -->Text after comments and PI</wrapper>
			trace(commentsAndPI);
			
			var emptyList1 = new XMLList();
			var emptyList2 = new XMLList("");
			trace("Empty lists: " + emptyList1.length() + " " + emptyList2.length());

			var trimmedXML = <a>  
			foo <b>  
			bar</b>
			</a>;
			XML.prettyPrinting = false;
			trace("trimmedXML: " + trimmedXML);			
			// FIXME - enable this when Ruffle throws coercion errors
			//XML.prototype.name.apply(5);
		}
	}
}

Test.run();
