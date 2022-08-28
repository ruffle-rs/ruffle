package {
	public class Test {}
}

class ES4Class {
	public static var INSTANCE: ES4Class = new ES4Class();
	
	public static var NULL: ES4Class = null;
	
	public static var UNDEFINED: ES4Class = undefined;
}

trace("///ES4Class.INSTANCE === null");
trace(ES4Class.INSTANCE === null);
trace("///ES4Class.INSTANCE === undefined");
trace(ES4Class.INSTANCE === undefined);
trace("///ES4Class.INSTANCE is ES4Class");
trace(ES4Class.INSTANCE is ES4Class);

trace("///ES4Class.NULL === null");
trace(ES4Class.NULL === null);
trace("///ES4Class.NULL === undefined");
trace(ES4Class.NULL === undefined);
trace("///ES4Class.NULL is ES4Class");
trace(ES4Class.NULL is ES4Class);

trace("///ES4Class.UNDEFINED === null");
trace(ES4Class.UNDEFINED === null);
trace("///ES4Class.UNDEFINED === undefined");
trace(ES4Class.UNDEFINED === undefined);
trace("///ES4Class.UNDEFINED is ES4Class");
trace(ES4Class.UNDEFINED is ES4Class);