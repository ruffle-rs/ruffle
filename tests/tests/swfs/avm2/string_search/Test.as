package {
	public class Test {}
}

var ruffle_object = {s: "Ruffle Test Object"};
ruffle_object.toString = function() {
    return this.s;
}

//Search tests
trace("// var str:String = new String(\"mtchablematmatmat\");");
var str:String = new String("mtchablematmatmat");
trace("// var ret:int = str.search(\"mat\");");
var ret:int = str.search("mat");
trace("// trace(ret);");
trace(ret);

trace("// var re = new RegExp(\"MA*T|a[a-z]*e\",\"i\");");
var re = new RegExp("MA*T|a[a-z]*e","i");
trace("// re.lastIndex = 3;");
re.lastIndex = 3;
trace("// trace(str.search(re), re.lastIndex);");
trace(str.search(re), re.lastIndex);
trace("// trace(str.search(re), re.lastIndex);");
trace(str.search(re), re.lastIndex);
trace("// trace(str.search(re), re.lastIndex);");
trace(str.search(re), re.lastIndex);

trace("// trace(str.search(new RegExp(\"MA*T|a[a-z]*e\",\"i\")));");
trace(str.search(new RegExp("MA*T|a[a-z]*e","i")));
trace("// trace(str.search(new RegExp(\"ma*t|a[a-z]*e\",\"\")));");
trace(str.search(new RegExp("ma*t|a[a-z]*e","")));
trace("// trace(str.search(new RegExp(\"ma*t|a[a-z]*e\",\"g\")));");
trace(str.search(new RegExp("ma*t|a[a-z]*e","g")));
trace("// trace(str.search(new RegExp(\"notmatch\", \"g\")));");
trace(str.search(new RegExp("notmatch", "g")));

trace("// var subject:String = \"AAA\";");
var subject:String = "AAA";
trace("// trace(subject.search(/(((((((((((((((((((a*)(abc|b))))))))))))))))))*.)*(...)*/g));");
trace(subject.search(/(((((((((((((((((((a*)(abc|b))))))))))))))))))*.)*(...)*/g));
trace("// trace(subject.search(/((((((((((((((((((d|.*)))))))))))))))))*.)*(...)*/g));");
trace(subject.search(/((((((((((((((((((d|.*)))))))))))))))))*.)*(...)*/g));
trace("// trace(subject.search(/((((((((((((((((((a+)*))))))))))))))))*.)*(...)*/g));");
trace(subject.search(/((((((((((((((((((a+)*))))))))))))))))*.)*(...)*/g));

trace("// trace(subject.search(\"((((((((((((((((((a+)*))))))))))))))))*.)*(...)*\"));");
trace(subject.search("((((((((((((((((((a+)*))))))))))))))))*.)*(...)*"));

trace("// trace(subject.search(\"((((((((((((((((((a+)*))))))))))))))))*.)*(...)*\"));");
trace(subject.search("((((((((((((((((((a+)*))))))))))))))))*.)*(...)*"));
trace("// trace(subject.search(\"(A)(A)\"));");
trace(subject.search("(A)(A)"));
trace("// trace(subject.search(\"AAA\"));");
trace(subject.search("AAA"));
trace("// trace(subject.search(\"AA\"));");
trace(subject.search("AA"));
trace("// trace(subject.search(\"A\"));");
trace(subject.search("A"));

trace("// trace(str.search(ruffle_object));");
trace(str.search(ruffle_object));
