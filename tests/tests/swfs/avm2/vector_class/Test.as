package {
	import flash.utils.getDefinitionByName;
	import flash.utils.getQualifiedClassName;
	public class Test {
		public function Test() {
			var vec;
			var name;
			
			vec = new Vector.<int>([1, 2]);
			name = getQualifiedClassName(vec);
			trace("Vector.<int> name: " + name);
			var klass = getDefinitionByName(name);
			trace("Vector.<int>: " + klass);
			
			vec = new Vector.<uint>([1, 2]);
			name = getQualifiedClassName(vec);
			trace("Vector.<uint> name: " + name);
			var klass = getDefinitionByName(name);
			trace("Vector.<uint>: " + klass);
			
			vec = new Vector.<Number>([1, 2]);
			name = getQualifiedClassName(vec);
			trace("Vector.<Number> name: " + name);
			var klass = getDefinitionByName(name);
			trace("Vector.<Number>: " + klass);	

			vec = new Vector.<Object>([1, 2]);
			name = getQualifiedClassName(vec);
			trace("Vector.<Object> name: " + name);
			var klass = getDefinitionByName(name);
			trace("Vector.<Object>: " + klass);
		}
	}
}