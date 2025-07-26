/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "JSON";
// var VERSION = "AS3";
// var TITLE   = "JSON adhoc tests for JSON.parse and JSON.stringify";


function removeExceptionDetail(s:String) {
    var fnd=s.indexOf(" ");
    if (fnd>-1) {
        if (s.indexOf(':',fnd)>-1) {
                s=s.substring(0,s.indexOf(':',fnd));
        }
    }
    return s;
}

function sortObject(o:Object) {
    var keys=[];
    var key;
    for ( key in o ) {
        if (o[key]===undefined) {
           continue;
        }
        keys[keys.length]=key;
    }
    keys.sort();
    var ret="{";
    var value;
    for (var i in keys) {
        key=keys[i];
        value=o[key];
        if (value is String) {
            value='"'+value+'"';
        } else if (value is Array) {
            value='['+value+']';
        } else if (value is Object) {
        }
        ret += '"'+key+'":'+value+",";
    }
    ret=ret.substring(0,ret.length-1);
    ret+="}";
    return ret;
}

function countSpaces(s:String) {
    var spaces:uint=0;
    for (var i=0;i<s.length;i++) {
        if (s.charAt(i)==' ') spaces+=1;
    }
    return spaces;
}

function jformat(x) {
    if (x is String)
    return '"' + x + '"';
    else
    return x;
}

// set the Object.toString and Array.toString to show each property value
// instead of [Object object] for testing
var oldObject = Object.prototype.toString;
var oldArray = Array.prototype.toString;
Object.prototype.toString =
    (function () {
    var s = "{";
    var first = true;
    for ( var i in this ) {
    if (this.hasOwnProperty(i)) {
        if (!first)
        s += ",";
        s += jformat(String(i)) + ":" + jformat(this[i]);
        first = false;
    }
    }
    return s + "}";
});
Array.prototype.toString =
    (function () {
    var s = "[";
    var first = true;
    for ( var i=0 ; i < this.length ; i++ ) {
        if (!first)
            s += ",";
        first = false;
        if (!this.hasOwnProperty(i))
            continue;
        s += jformat(this[i]);
    }
    return s + "]";
});

// Assert.expectEq(comment,expected,actual)
// test JSON.parse on numeric values
Assert.expectEq("JSON.parse(1)",1,JSON.parse("1"));
Assert.expectEq("JSON.parse(-1.75e12)","-1750000000000",JSON.parse("-1.75e12").toString());
Assert.expectEq("JSON.parse(-1.75)",-1.75,JSON.parse("-1.75"));
Assert.expectEq("JSON.parse(1.75)",1.75,JSON.parse("1.75"));
Assert.expectEq("JSON.parse(-1e-12)","-1e-12",JSON.parse("-1e-12").toString())
Assert.expectEq("JSON.parse(-1e+12)","-1000000000000",JSON.parse("-1e+12").toString());

// test JSON.parse on simple string
Assert.expectEq("JSON.parse(supercallifragilistic)","supercallifragilistic",JSON.parse("\"supercallifragilistic\""));

// test JSON.parse on null
Assert.expectEq("JSON.parse(null)",null,JSON.parse("  null "));

// test JSON.parse true/false
Assert.expectEq("JSON.parse(true)",true,JSON.parse("true"));
Assert.expectEq("JSON.parse(false)",false,JSON.parse("\t\n\rfalse"));

// test JSON.parse array
Assert.expectEq("JSON.parse([1,2,3,true,false,null])","[1,2,3,true,false,null]",JSON.parse("[1,2,3,true,false,null]").toString());

// test JSON.parse object
Assert.expectEq("JSON.parse({a:10,b:20})","{\"a\":10,\"b\":20}",sortObject(JSON.parse("{\"a\":10,\"b\":20}")));
Assert.expectEq("JSON.parse({c:[1,2,3]}},a:{b:20})",'{"a":{"b":20},"c":[[1,"2",3]]}',sortObject(JSON.parse("{\"a\":{\"b\":20},\"c\":[1,\"2\",3]}")));

// test JSON.parse revivers
Assert.expectEq("JSON.parse(1,reviver)","44",JSON.parse("1", function (name, val) { return "44" }));
Assert.expectEq("JSON.parse([1,-2,3],negate reviver)","[-1,2,-3]",JSON.parse("[1,-2,3]", function (name, val) { if (val is Number) return -val; else return val; }).toString());
Assert.expectEq("JSON.parse([1,-2,3],undef neg reviver)",JSON.parse("[1,-2,3]", function (name, val) { if (val is Number && val < 0) return undefined; else return val; }).toString(),"[1,,3]");
// not sure why?
Assert.expectEq("JSON.parse([1,-2,3],undef 1 reviver)","[\"1\",,3]",JSON.parse("[\"1\",-2,3]", function (name, val) { if (name == "1") return undefined; else return val; }).toString());
Assert.expectEq("JSON.parse({a:{\"1\":20},c:[1,2,3]})","{\"a\":{\"1\":20},\"c\":[[,\"2\",3]]}",
  sortObject(JSON.parse("{\"a\":{\"1\":20},\"c\":[1,\"2\",3]}", function (name, val) { if (val == "1") return undefined; else return val; })));

// reset Object and Array toString to initial values
Object.prototype.toString = oldObject;
Array.prototype.toString = oldArray;

// test JSON.stringify on numbers
Assert.expectEq("JSON.stringify(1)","1",JSON.stringify(JSON.parse("1")));
Assert.expectEq("JSON.stringify(-1.75e12)","-1750000000000",JSON.stringify(JSON.parse("-1.75e12")));
Assert.expectEq("JSON.stringify(-1.75)","-1.75",JSON.stringify(JSON.parse("-1.75")));
Assert.expectEq("JSON.stringify(1.75)","1.75",JSON.stringify(JSON.parse("1.75")));
Assert.expectEq("JSON.stringify(-1e-12)","-1e-12",JSON.stringify(JSON.parse("-1e-12")));
Assert.expectEq("JSON.stringify(-1e+12)","-1000000000000",JSON.stringify(JSON.parse("-1e+12")));

// test JSON.stringify on string
Assert.expectEq("JSON.stringify(supercallifragilistic)","\"supercallifragilistic\"",JSON.stringify(JSON.parse("\"supercallifragilistic\"")));

// test JSON.stringify on null
Assert.expectEq("JSON.stringify(null)","null",JSON.stringify(JSON.parse("  null ")));

// test JSON.stringify on booleans
Assert.expectEq("JSON.stringify(true)","true",JSON.stringify(JSON.parse("true")));
Assert.expectEq("JSON.stringify(false)","false",JSON.stringify(JSON.parse("\t\n\rfalse")));

// test JSON.stringify on array
Assert.expectEq("JSON.stringify(array)","[1,2,3,true,false,null]",JSON.stringify(JSON.parse("[1,2,3,true,false,null]")));

// test JSON.stringify on objects
Assert.expectEq("JSON.stringify(object)","{\"a\":10,\"b\":20}",sortObject(JSON.parse(JSON.stringify(JSON.parse("{\"a\":10,\"b\":20}")))));
Assert.expectEq("JSON.stringify(nested object)",'{"a":[object Object],"c":[1,2,3]}',sortObject(JSON.parse("{\"a\":{\"b\":20},\"c\":[1,\"2\",3]}")));

// todo: why are the \n appearing? in all space
// test JSON.stringify with space=string
Assert.expectEq("JSON.stringify(array,replacer=null,space)","[\n 1,\n 2,\n 3,\n true,\n false,\n null\n]",JSON.stringify(JSON.parse("[1,2,3,true,false,null]"),null," "));
Assert.expectEq("JSON.stringify(object,replacer=null,space)",15,countSpaces(JSON.stringify(JSON.parse("{\"a\":{\"b\":20},\"c\":[1,\"2\",3]}"), null," ")));

// test JSON.stringify with space as Array
Assert.expectEq("JSON.stringify(array,replacer=array)","[1,2,3,true,false,null]",JSON.stringify(JSON.parse("[1,2,3,true,false,null]"),[1,3,5]));
Assert.expectEq("JSON.stringify(object,replacer=array)","{\"a\":{},\"c\":[1,\"2\",3]}",JSON.stringify(JSON.parse("{\"a\":{\"b\":20},\"c\":[1,\"2\",3]}"), ["a","c"]));

// test JSON.stringify with space as function
Assert.expectEq("JSON.stringify(array,replacer=function)","[1,null,3,null,false,null]",JSON.stringify(JSON.parse("[1,2,3,true,false,null]"), function (key, value) { if (!(parseInt(key) & 1)) return value; return undefined; } ));
Assert.expectEq("JSON.stringify(object,replacer=function)","{\"a\":[object Object],\"c\":[-1,2,-3]}",sortObject(JSON.parse(JSON.stringify(JSON.parse("{\"a\":{\"b\":20},\"c\":[1,\"2\",3]}")), function (key, value) { if (value is Number) return -value; return value; } )));


// test JSON.stringify circular structures throw TypeError exception
negativeTestException="no exception thrown";
try {
    var a= [];
    a[0]=a;
    JSON.stringify(a);
} catch(e) {
    negativeTestException=e.toString();
    negativeTestException=removeExceptionDetail(negativeTestException);
}
Assert.expectEq("JSON.stringify(circular structure)","TypeError: Error #1129",negativeTestException);

Assert.expectEq("test Quote on escaped characters: JSON.stringify(\\b,\\t,\\f,\\r)",'"\\b,\\t,\\f,\\r"',JSON.stringify("\b,\t,\f,\r"));
Assert.expectEq("test Quote on double quote",'"\\""',JSON.stringify('\"'));


Object.prototype.toString=oldObject;
Array.prototype.toString=oldArray;
