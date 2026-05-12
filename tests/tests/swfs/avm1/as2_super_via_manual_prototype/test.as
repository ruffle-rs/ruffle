trace("First frame");
trace("");

trace("// Base.prototype = new MovieClip();");
Base.prototype = new MovieClip();
trace("");

trace("// Extended.prototype = new Base();");
Extended.prototype = new Base();
trace("");

trace("// ExtendedFurther.prototype = new Extended();");
ExtendedFurther.prototype = new Extended();
trace("");


trace("// var ef = new ExtendedFurther();");
var ef = new ExtendedFurther();
trace("");

trace("// ef.test_method();");
ef.test_method();
trace("");

trace("// trace(ef.test_property);");
trace(ef.test_property);
trace("");

trace("// ef.test_property = \"\";");
ef.test_property = "";
trace("");

fscommand("quit");
