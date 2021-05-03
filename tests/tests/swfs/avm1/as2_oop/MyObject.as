import MyInterface;

class MyObject implements MyInterface, MyOtherInterface {
	function a() {
		trace("MyObject.a called");
	}
	
	function b() {
		trace("clock crew's back baby");
	}
	
	function c() {
		trace("MyObject.c called");
	}
}