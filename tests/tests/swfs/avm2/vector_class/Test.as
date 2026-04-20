package {
	import flash.utils.getDefinitionByName;
	import flash.utils.getQualifiedClassName;
	import flash.system.ApplicationDomain;
	

	
	public class Test {
		public function Test() {
			var vec;
			var name;
			
			vec = new Vector.<int>([1, 2]);
			name = getQualifiedClassName(vec);
			trace("Vector.<int> name: " + name);
			trace("Vector.<int>: " + getDefinitionByName(name));
			trace("ApplicationDomain.hasDefinition Vector.<int>: " + ApplicationDomain.currentDomain.hasDefinition(name));
			trace("ApplicationDomain.getDefinition Vector.<int>: " + ApplicationDomain.currentDomain.getDefinition(name));
			
			vec = new Vector.<uint>([1, 2]);
			name = getQualifiedClassName(vec);
			trace("Vector.<uint> name: " + name);
			trace("Vector.<uint>: " + getDefinitionByName(name));
			trace("ApplicationDomain.hasDefinition Vector.<uint>: " + ApplicationDomain.currentDomain.hasDefinition(name));
			trace("ApplicationDomain.getDefinition Vector.<uint>: " + ApplicationDomain.currentDomain.getDefinition(name));
			
			vec = new Vector.<Number>([1, 2]);
			name = getQualifiedClassName(vec);
			trace("Vector.<Number> name: " + name);
			trace("Vector.<Number>: " + getDefinitionByName(name));
			trace("ApplicationDomain.hasDefinition Vector.<Number>: " + ApplicationDomain.currentDomain.hasDefinition(name));
			trace("ApplicationDomain.getDefinition Vector.<Number>: " + ApplicationDomain.currentDomain.getDefinition(name));

			trace("Early lookup: " + ApplicationDomain.currentDomain.hasDefinition("__AS3__.vec::Vector.<Test>"));
			vec = new Vector.<Object>([1, 2]);
			name = getQualifiedClassName(vec);
			trace("Vector.<Object> name: " + name);
			trace("Vector.<Object>: " + getDefinitionByName(name));
			trace("ApplicationDomain.hasDefinition Vector.<Object>: " + ApplicationDomain.currentDomain.hasDefinition(name));
			trace("ApplicationDomain.getDefinition Vector.<Object>: " + ApplicationDomain.currentDomain.getDefinition(name));
			
			vec = new Vector.<Test>([]);
			name = getQualifiedClassName(vec);
			trace("Vector.<Test> name: " + name);
			trace("Vector.<Test>: " + getDefinitionByName(name));
			trace("ApplicationDomain.hasDefinition Vector.<Test>: " + ApplicationDomain.currentDomain.hasDefinition(name));
			trace("ApplicationDomain.getDefinition Vector.<Test>: " + ApplicationDomain.currentDomain.getDefinition(name));

			import rs.ruffle.CustomClass;
			
			trace("Vector.<rs.ruffle.CustomClass>: " + getDefinitionByName("Vector.<rs.ruffle.CustomClass>"));
			trace("__AS3__.vec::Vector.<rs.ruffle.CustomClass>: " + getDefinitionByName("__AS3__.vec::Vector.<rs.ruffle.CustomClass>"));
			
			vec = new Vector.<CustomClass>([]);	
		
			name = getQualifiedClassName(vec);
			trace("Vector.<CustomClass> name: " + name);
			trace("Vector.<CustomClass>: " + getDefinitionByName(name));
			trace("ApplicationDomain.hasDefinition Vector.<CustomClass>: " + ApplicationDomain.currentDomain.hasDefinition(name));
			trace("ApplicationDomain.getDefinition Vector.<CustomClass>: " + ApplicationDomain.currentDomain.getDefinition(name));

			vec = new Vector.<Vector.<int>>();
			name = getQualifiedClassName(vec);
			trace("Vector.<Vector.<int>> name: " + name);
			trace("__AS3__.vec::Vector.<__AS3__.vec::Vector.<int>>: " + getDefinitionByName(name));
			trace("ApplicationDomain.hasDefinition __AS3__.vec::Vector.<__AS3__.vec::Vector.<int>>: " + ApplicationDomain.currentDomain.hasDefinition(name));
			trace("ApplicationDomain.getDefinition __AS3__.vec::Vector.<__AS3__.vec::Vector.<int>>: " + ApplicationDomain.currentDomain.getDefinition(name));
			name = "Vector.<Vector.<int>>";
			trace("Vector.<Vector.<int>>: " + getDefinitionByName(name));
			trace("ApplicationDomain.hasDefinition Vector.<Vector.<int>>: " + ApplicationDomain.currentDomain.hasDefinition(name));
			trace("ApplicationDomain.getDefinition Vector.<Vector.<int>>: " + ApplicationDomain.currentDomain.getDefinition(name));
			trace("Vector.<Number> without namespace" + getDefinitionByName("Vector.<Number>"));

			trace("Vector.<Number> without namespace" + getDefinitionByName("Vector.<Number>"));
			try {
				trace("Vector without namespace: " + getDefinitionByName("Vector"));
			} catch (e) {
				trace("Caught error: " + e);
			}
			
		}
	}
}