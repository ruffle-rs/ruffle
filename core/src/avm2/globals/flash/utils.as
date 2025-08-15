package flash.utils {

    public native function getDefinitionByName(name:String):Object;
    public native function getQualifiedClassName(value:*):String;
    public native function getQualifiedSuperclassName(value:*):String;

    [Ruffle(FastCall)]
    public native function getTimer():int;

    public function describeType(value:*): XML {
        // TODO: Also set @alias on the resulting XML
        if (value === undefined) {
            // avmplus throws this error from the alias-lookup code,
            // which we don't currently have implemented
            throw new TypeError("Error #1010: A term is undefined and has no properties.", 1010);
        }

        return avmplus.describeType(value, avmplus.FLASH10_FLAGS);
    }

    public native function setInterval(closure:Function, delay:Number, ... arguments):uint;
    public native function clearInterval(id:uint):void;
    public native function setTimeout(closure:Function, delay:Number, ... arguments):uint;
    public native function clearTimeout(id:uint):void;
    public native function escapeMultiByte(s:String):String;
    public native function unescapeMultiByte(s:String):String;
}
