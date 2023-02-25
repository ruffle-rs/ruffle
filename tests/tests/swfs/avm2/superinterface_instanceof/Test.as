package {
	public class Test {
		public static function test() {
			var baseObj = new BaseClass();
			var interfaces = [FirstInterface, FirstInterfaceParent, FirstInterfaceGrandparent,SecondInterface, SecondInterfaceParent, SecondInterfaceGrandparent, SuperInterface, SuperInterfaceParent, SuperInterfaceGrandparent];
			for each (var klass in interfaces) {
				trace("baseObj is " + klass + ": " + (baseObj is klass));
			}
		
			var superObj = new SuperClass();
			for each (var klass in interfaces) {
				trace("superObj is " + klass + ": " + (superObj is klass));
			}
		}
	}
}

class SuperClass implements SuperInterface {}
class BaseClass extends SuperClass implements FirstInterface, SecondInterface {}

interface FirstInterface extends FirstInterfaceParent {}
interface FirstInterfaceParent extends FirstInterfaceGrandparent {}
interface FirstInterfaceGrandparent {}

interface SecondInterface extends FirstInterfaceParent, SecondInterfaceParent {}
interface SecondInterfaceParent extends SecondInterfaceGrandparent {}
interface SecondInterfaceGrandparent {}

interface SuperInterface extends SuperInterfaceParent, SecondInterfaceParent {}
interface SuperInterfaceParent extends SuperInterfaceGrandparent {}
interface SuperInterfaceGrandparent {}

