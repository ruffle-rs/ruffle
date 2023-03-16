package {
	public class Test {
		public function Test() {
			XML.prettyPrinting = false;
			
			var xml:XML = 
				  <enrollees>
					<student id="239">
						<myClass name="Algebra" />
						<myClass name="Spanish 2"/>
					</student>
					<student id="206">
						<myClass name="Trigonometry" />
						<myClass name="Spanish 2" />
					</student>
				  </enrollees>;
			
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
		}
	}
}