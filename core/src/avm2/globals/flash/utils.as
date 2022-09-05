package flash.utils {
	public native function getDefinitionByName(name:String):Object;
	public native function getQualifiedClassName(value:*):String;
	public native function getQualifiedSuperclassName(value:*):String;
	public native function getTimer():int;

	public native function setInterval(closure:Function, delay:Number, ... arguments):uint;
	public native function clearInterval(id:uint):void;
	public native function setTimeout(closure:Function, delay:Number, ... arguments):uint;
	public native function clearTimeout(id:uint):void;
	public native function escapeMultiByte(s:String):String;
}
