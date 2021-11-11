package {
	public class Test {}
}

function fn() {
}

class Cls {
	public function method() {
		
	}
	
	public static function static_method() {
		
	}
}

trace("///fn is Function");
trace(fn is Function);

trace("///new Cls().method is Function");
trace(new Cls().method is Function);

trace("///Cls.static_method is Function");
trace(Cls.static_method is Function);