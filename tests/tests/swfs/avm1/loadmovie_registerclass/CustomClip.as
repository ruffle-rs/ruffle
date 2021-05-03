class CustomClip extends MovieClip {
	function CustomClip() {
		super();
		test();
	}
	
	function test() {
		trace("// this.box");
		trace(this['box']);
		trace("");
	}
}