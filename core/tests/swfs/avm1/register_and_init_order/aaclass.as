class aaclass extends MovieClip {
	function aaclass() {
		trace("aaclass constructor");
		super();
		
		trace("");
		trace("// trace(this._name)");
		trace(this._name);
		trace("");
		
		this.test();
		trace("aaclass constructor end");
		trace("");
		trace("");
	}
	
	function test() {
		trace("aaclass test()");
		
		trace("");
		
		trace("// trace(this._name)");
		trace(this["_name"]);
		trace("");
		
		trace("// trace(this.box)");
		trace(this["box"]);
		trace("");
		
		trace("aaclass test() end");
	}
}