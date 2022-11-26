package {
	public class Test {
		public function Test() {}
	}
}

class MyClass {
	public const pubConst:String = "Public const";
	protected const protectedConst:String = "Protected const";
	private const privConst:String = "Private const";
	
	public static var pubVar:String = "Public static variable";
	protected static var protectedVar:String = "Protected static variable";
	private static var privVar:Boolean = false;
	
	public static function pubMethod() {}
	protected static function protectedMethod() {}
	private static function privMethod() {}
}

for each (var key in MyClass) {
	trace("Key: " + key + " Value: " + MyClass[key]);
}

var obj:Class = MyClass;
trace("pubConst: " + obj["pubConst"]);
trace("protectedConst: " + obj["protectedConst"]);
trace("privConst: " + obj["privConst"]);
trace("pubVar: " + obj["pubVar"]);
trace("protectedVar: " + obj["protectedVar"]);
trace("privVar: " + obj["privVar"]);
trace("pubMethod: " + obj["pubMethod"]);
trace("protectedMethod: " + obj["protectedMethod"]);
trace("privMethod: " + obj["privMethod"]);

trace('obj.prototype["pubVar"]: ' + obj.prototype["pubVar"]);