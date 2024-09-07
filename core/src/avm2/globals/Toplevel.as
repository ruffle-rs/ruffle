package {
    public namespace AS3 = "http://adobe.com/AS3/2006/builtin";

    public const NaN: Number = 0 / 0;

    public const Infinity: Number = 1 / 0;

    public const undefined = void 0;

    public native function encodeURI(uri:String = "undefined"):String;
    public native function encodeURIComponent(uri:String = "undefined"):String;

    public native function decodeURI(uri:String = "undefined"):String;
    public native function decodeURIComponent(uri:String = "undefined"):String;

    public native function escape(string:String = "undefined"):String;
    public native function unescape(string:String = "undefined"):String;

    public native function isXMLName(string:* = undefined):Boolean;

    public native function isFinite(value:Number = undefined):Boolean;
    public native function isNaN(value:Number = undefined):Boolean;

    public native function parseFloat(number:String = "NaN"):Number;
    public native function parseInt(string:String = "NaN", base:int = 0):Number;

    public native function trace(... rest):void;
}

// These classes are required by other core code, so we put them here. Toplevel.as
// is loaded before the rest of the global code.

include "Error.as"

include "ArgumentError.as"
include "DefinitionError.as"
include "EvalError.as"
include "TypeError.as"
include "RangeError.as"
include "ReferenceError.as"
include "SecurityError.as"
include "SyntaxError.as"
include "UninitializedError.as"
include "URIError.as"
include "VerifyError.as"

include "JSON.as"
include "Namespace.as"
include "QName.as"
include "XML.as"
include "XMLList.as"
