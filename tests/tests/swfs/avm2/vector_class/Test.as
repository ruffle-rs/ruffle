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
		}
	}
}