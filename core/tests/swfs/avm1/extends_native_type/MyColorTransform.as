class MyColorTransform extends flash.geom.ColorTransform {
	function MyColorTransform() {
		super();
		trace("// MyColorTransform constructor");
		trace("// redMultiplier");
		trace(redMultiplier);
		trace("// redOffset");
		trace(redOffset);
		trace("// redOffset = 100");
		redOffset = 100;
		trace(redOffset);
	}
}