class CustomClass {
	var customProperty = "Custom property from CustomClass";
	
	function CustomClass() {
		trace("/// CustomClass() constructor start");
		trace("");
		
		trace("// this._name");
		trace(this["_name"]);
		trace("");
		
		trace("// this.customProperty");
		trace(this["customProperty"]);
		trace("");
		
		trace("// this.child");
		trace(this["child"]);
		trace("");
		
		trace("/// CustomClass() constructor end");
		trace("");
	}
}