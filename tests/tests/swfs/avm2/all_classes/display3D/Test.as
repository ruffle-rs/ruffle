package {
    import avmplus.FLASH10_FLAGS;
    import avmplus.INCLUDE_METADATA;
    import avmplus.describeType;
    import flash.display.Sprite;
    import flash.utils.getDefinitionByName;

    public class Test extends Sprite {
        static const CLASSES:Array = ["Context3D","Context3DBlendFactor","Context3DBufferUsage","Context3DClearMask","Context3DCompareMode","Context3DMipFilter","Context3DProfile","Context3DProgramType","Context3DRenderMode","Context3DStencilAction","Context3DTextureFilter","Context3DTextureFormat","Context3DTriangleFace","Context3DVertexBufferFormat","Context3DWrapMode","IndexBuffer3D","Program3D","VertexBuffer3D"];

        public function Test() {
            super();
            for(var i in CLASSES) {
                var klass:Class;
                try {
                    klass = lookupClass(CLASSES[i]);
                }
                catch(e:Error) {
                    trace(CLASSES[i] + " not accessible");
                    continue;
                }
                var described:XML = describeType(klass, FLASH10_FLAGS & ~INCLUDE_METADATA);
                trace(normalizeXML(described));
            }
        }

        public function lookupClass(className:String):Class {
            return getDefinitionByName("flash.display3D::" + className);
        }

        public function normalizeXML(data: XML, indent:uint = 0) {
	        var output = "";
	        for (var i = 0; i < indent; i ++) {
		        output += " ";
	        };
	        output += "<" + data.name();
	        for each (var attr in data.attributes()) {
		        output += " " + attr.name() + "=\"" + attr + "\"";
	        }
	        if (data.children().length() == 0) {
		        output += "/>";
		        return output;
	        }
	        output += ">\n";
	        var childStrs = [];
	        for each (var child in data.children()) {
		        childStrs.push(normalizeXML(child, indent + 2));
	        }
	        childStrs.sort();
	        for each (var childStr in childStrs) {
		        for (var i = 0 ; i < indent; i ++) {
			        output += " ";
		        }
		        output += childStr;
		        output += "\n"
	        }
	        for (var i = 0; i < indent; i ++) {
		        output += " ";
	        };
	        output += "</" + data.name() + ">";
	        return output;
        }
    }
}
