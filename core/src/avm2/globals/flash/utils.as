package flash.utils {
	public native function getDefinitionByName(name:String):Object;
	public native function getQualifiedClassName(value:*):String;
	public native function getQualifiedSuperclassName(value:*):String;
	public native function getTimer():int;

	// note: this is an extremely silly hack,
	// made specifically to fool com.adobe.serialization.json.JsonEncoder.
	// this relies on the fact that a.@b in Ruffle is unimplemented and behaves like a.b.
	// once we get proper XML support, this entire impl is to be trashed.
	public function describeType(value:*): XML {
		import __ruffle__.stub_method;
		stub_method("flash.utils", "describeType");

		var ret = new XML();
		ret.name = getQualifiedClassName(value);
		return ret;
	}

	public native function setInterval(closure:Function, delay:Number, ... arguments):uint;
	public native function clearInterval(id:uint):void;
	public native function setTimeout(closure:Function, delay:Number, ... arguments):uint;
	public native function clearTimeout(id:uint):void;
	public native function escapeMultiByte(s:String):String;
	public native function unescapeMultiByte(s:String):String;
}
