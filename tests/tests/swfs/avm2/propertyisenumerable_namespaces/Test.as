package {
	public class Test {}
}

dynamic class ES4Class {
	AS3 function as3_prop() {
		
	}
}

trace("///new ES4Class.propertyIsEnumerable(\"as3_prop\");");
trace(new ES4Class().propertyIsEnumerable("as3_prop"));

trace("///new ES4Class.setPropertyIsEnumerable(\"as3_prop\", true);");
trace(new ES4Class().setPropertyIsEnumerable("as3_prop", true));

trace("///new ES4Class.propertyIsEnumerable(\"as3_prop\");");
trace(new ES4Class().propertyIsEnumerable("as3_prop"));