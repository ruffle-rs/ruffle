package {
	import flash.display.Sprite;

	public class Test extends Sprite {
		public function Test() {
			XML.prettyPrinting = false;

			var xml:XML =
				  <enrollees>
					<student id="239">
						<myClass name="Algebra" student="Algebra Student attribute" />
						<myClass name="Spanish 2"/>
					</student>
					<student id="206">
						<myClass name="Trigonometry" student="Trigonometry Student attribute" />
						<myClass name="Spanish 2" />
					</student>
				  </enrollees>;

				trace("xml..@student : " + xml..@student);
				trace("xml..@* : " + xml..@*);

				trace("myClass descendants:");
				trace(xml.descendants("myClass"));

				trace("* descendants:");
				trace(xml.descendants("*"));

				trace("No-arg descendants:");
				trace(xml.descendants());

				trace("student descendants operator:");
				trace(xml..student);

				trace("myClass descendants operator:");
				trace(xml..myClass);

				trace("Non-matching descendants");
				trace(xml.descendants("missingName"));

				trace("Non-matching descendants operator");
				trace(xml..missingName);

				var list = new XMLList(xml.toXMLString() + "<myClass name='TopLevel'><inner><myClass name = 'Nested'></myClass></inner></myClass>");

				trace("myClass list descendants");
				trace(list.descendants("myClass"));

				trace("list * descendants:");
				trace(list.descendants("*"));

				trace("list no-arg descendants;");
				trace(list.descendants());

				trace("myClass list descendants operator");
				trace(list..myClass);

				trace("Non-matching list descendants");
				trace(list.descendants("missingName"));

				trace("Non-matching list descendants operator");
				trace(list..missingName);

				// Default namespace tests
				var nsXml:XML = <mesh xmlns="http://example.com"><vertices id="v1"><vertex/></vertices></mesh>;

				trace("ns xml without default namespace:");
				trace("nsXml..vertices length: " + nsXml..vertices.length());

				default xml namespace = new Namespace("http://example.com");
				trace("ns xml with default namespace set:");
				trace("nsXml..vertices length: " + nsXml..vertices.length());
				trace("nsXml..vertices.@id: " + nsXml..vertices.@id);

				trace("ns xml direct child access:");
				trace("nsXml.vertices length: " + nsXml.vertices.length());

				default xml namespace = "";
				trace("ns xml after reset:");
				trace("nsXml..vertices length: " + nsXml..vertices.length());

				var ns:Namespace = new Namespace("http://example.com");
				trace("ns xml explicit namespace:");
				trace("nsXml..ns::vertices length: " + nsXml..ns::vertices.length());

				default xml namespace = new Namespace("http://example.com");
				var nsXml2:XML = <root xmlns="http://example.com"><level1><level2><target id="deep"/></level2></level1></root>;
				trace("ns xml nested descendants:");
				trace("nsXml2..target length: " + nsXml2..target.length());
				trace("nsXml2..target.@id: " + nsXml2..target.@id);

				trace("ns XMLList descendants:");
				var nsList:XMLList = nsXml2.level1;
				trace("nsList..target length: " + nsList..target.length());
		}
	}
}
