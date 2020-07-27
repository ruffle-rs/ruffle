class aaclass extends MovieClip {
	function aaclass() {
		super();
		trace("aaclass constructor");
		
		trace("");
		trace("// trace(this._name)");
		trace(this._name);
		trace("");
		trace("// trace(this.__constructor__)");
		trace(this["__constructor__"]);
		trace("");
		trace("// trace(this.__constructor__ === aaclass)");
		trace(this["__constructor__"] === aaclass);
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