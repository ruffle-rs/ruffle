package {
	public class test {}
}

var ruffle_object = {s: "Ruffle Test Object"};
ruffle_object.toString = function() {
    return this.s;
}

//Match tests
trace("// var str:String = new String(\"matchablematmatmat\");");
var str:String = new String("matchablematmatmat");
trace("// var ret:Array = str.match(\"mat\");");
var ret:Array = str.match("mat");
trace("// trace(ret);");
trace(ret);

trace("// var re = new RegExp(\"MA*T|a[a-z]*e\",\"i\");");
var re = new RegExp("MA*T|a[a-z]*e","i");
trace("// re.lastIndex = 3;");
re.lastIndex = 3;
trace("// trace(str.match(re), re.lastIndex);");
trace(str.match(re), re.lastIndex);
trace("// trace(str.match(re), re.lastIndex);");
trace(str.match(re), re.lastIndex);
trace("// trace(str.match(re), re.lastIndex);");
trace(str.match(re), re.lastIndex);

trace("// trace(str.match(new RegExp(\"MA*T|a[a-z]*e\",\"i\")));");
trace(str.match(new RegExp("MA*T|a[a-z]*e","i")));
trace("// trace(str.match(new RegExp(\"ma*t|a[a-z]*e\",\"\")));");
trace(str.match(new RegExp("ma*t|a[a-z]*e","")));
trace("// trace(str.match(new RegExp(\"ma*t|a[a-z]*e\",\"g\")));");
trace(str.match(new RegExp("ma*t|a[a-z]*e","g")));
trace("// trace(str.match(new RegExp(\"notmatch\", \"g\")));");
trace(str.match(new RegExp("notmatch", "g")));

trace("// var subject:String = \"AAA\";");
var subject:String = "AAA";
trace("// trace(subject.match(/(((((((((((((((((((a*)(abc|b))))))))))))))))))*.)*(...)*/g));");
trace(subject.match(/(((((((((((((((((((a*)(abc|b))))))))))))))))))*.)*(...)*/g));
trace("// trace(subject.match(/((((((((((((((((((d|.*)))))))))))))))))*.)*(...)*/g));");
trace(subject.match(/((((((((((((((((((d|.*)))))))))))))))))*.)*(...)*/g));
trace("// trace(subject.match(/((((((((((((((((((a+)*))))))))))))))))*.)*(...)*/g));");
trace(subject.match(/((((((((((((((((((a+)*))))))))))))))))*.)*(...)*/g));

trace("// trace(subject.match(\"((((((((((((((((((a+)*))))))))))))))))*.)*(...)*\"));");
trace(subject.match("((((((((((((((((((a+)*))))))))))))))))*.)*(...)*"));

trace("// trace(subject.match(\"((((((((((((((((((a+)*))))))))))))))))*.)*(...)*\"));");
trace(subject.match("((((((((((((((((((a+)*))))))))))))))))*.)*(...)*"));
trace("// trace(subject.match(\"(A)(A)\"));");
trace(subject.match("(A)(A)"));
trace("// trace(subject.match(\"AAA\"));");
trace(subject.match("AAA"));
trace("// trace(subject.match(\"AA\"));");
trace(subject.match("AA"));
trace("// trace(subject.match(\"A\"));");
trace(subject.match("A"));

trace("// trace(str.match(ruffle_object));");
trace(str.match(ruffle_object));
