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
// var TITLE   = "JSON adhoc tests for invalid input";

function removeExceptionDetail(s:String) {
    var fnd=s.indexOf(" ");
    if (fnd>-1) {
        if (s.indexOf(':',fnd)>-1) {
                s=s.substring(0,s.indexOf(':',fnd));
        }
    }
    return s;
}


// Assert.expectEq(comment,expected,actual)
exception1='no exception';
try {
    JSON.parse('-');
} catch (e) {
     exception1=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON numerical input '-'","SyntaxError: Error #1132",exception1);
exception2='no exception';
try {
    JSON.parse('.');
} catch (e) {
 exception2=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON numerical input '.'","SyntaxError: Error #1132",exception2);

exception3='no exception';
try {
    JSON.parse('1e');
} catch (e) {
 exception3=removeExceptionDetail(e.toString());
}
Assert.expectEq("valid JSON numerical input '1e'","SyntaxError: Error #1132",exception3);

exception4='no exception';
try {
    JSON.parse('1E');
} catch (e) {
 exception4=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON numerical input '1E'","SyntaxError: Error #1132",exception4);
exception5='no exception';
try {
    JSON.parse('1.e');
} catch (e) {
 exception5=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON numerical input '1.e'","SyntaxError: Error #1132",exception5);
exception6='no exception';
try {
    JSON.parse('\"\\');
} catch (e) {
 exception6=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON numerical input '\\'","SyntaxError: Error #1132",exception6);
exception7='no exception';
try {
    JSON.parse('"\\u"');
} catch (e) {
 exception7=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON unicode '\\u'","SyntaxError: Error #1132",exception7);
exception8='no exception';
try {
    JSON.parse('"\\uG000"');
} catch (e) {
 exception8=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON unicode '\\uG000'","SyntaxError: Error #1132",exception8);
exception9='no exception';
try {
    JSON.parse('"\\u0G00"');
} catch (e) {
 exception9=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON unicode '\\u0G00'","SyntaxError: Error #1132",exception9);
exception10='no exception';
try {
    JSON.parse('"\\u00G0"');
} catch (e) {
 exception10=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON unicode '\\u00G0'","SyntaxError: Error #1132",exception10);
exception11='no exception';
try {
    JSON.parse('"\\u000G"');
} catch (e) {
 exception11=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON unicode '\\u000G '","SyntaxError: Error #1132",exception11);
/*
exception12='no exception';
try {
    JSON.parse('"\\u0033');
} catch (e) {
 exception12=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON unicode '\\u0033' (missing end quote)","SyntaxError: Error #1132",exception12);
*/

// Bugzilla 672484: The string literal "\u000FF " *is* valid JSON input.
/*
exception13='no exception';
try {
    JSON.parse('"\\u000FF "');
} catch (e) {
 exception13=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON unicode '\\u000F'","SyntaxError: Error #1132",exception13);
*/

exception13='no exception';
try {
    JSON.parse('"\\u00bg"');
} catch (e) {
 exception13=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid JSON unicode '\\u00bg'","SyntaxError: Error #1132",exception13);

exception14='no exception';
try {
    JSON.parse('"\\z"');
} catch (e) {
 exception14=removeExceptionDetail(e.toString());
}
Assert.expectEq("Invalid escape character \\z","SyntaxError: Error #1132",exception14);

Assert.expectEq("JSON valid input: Escaped Quotes","\"",JSON.parse("\"\\\"\""));
Assert.expectEq("JSON lowercase hex digits","JOKN",JSON.parse('"\u004a\u004f\u004b\u004e"'));
Assert.expectEq("JSON lowercase hex digits","JOKN",JSON.parse('"\u004A\u004F\u004B\u004E"'));

var v1=undefined;
Assert.expectEq("JSON stringify undefined","null",JSON.stringify(v1));

var exception15='no exception';
try {
    JSON.parse('nzll');
} catch (e) {
 exception15=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'nzll'","SyntaxError: Error #1132",exception15);
var exception16='no exception';
try {
    JSON.parse('nuzl');
} catch (e) {
 exception16=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'nuzl'","SyntaxError: Error #1132",exception16);
var exception17='no exception';
try {
    JSON.parse('nulz');
} catch (e) {
 exception17=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'nulz'","SyntaxError: Error #1132",exception17);

// close to true
var exception18='no exception';
try {
    JSON.parse('tzue');
} catch (e) {
 exception18=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'tzue'","SyntaxError: Error #1132",exception18);
var exception19='no exception';
try {
    JSON.parse('trze');
} catch (e) {
 exception19=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'trze'","SyntaxError: Error #1132",exception19);
var exception20='no exception';
try {
    JSON.parse('truz');
} catch (e) {
 exception20=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'truz'","SyntaxError: Error #1132",exception20);

var exception21='no exception';
try {
    JSON.parse('fzlse');
} catch (e) {
 exception21=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'fzlse'","SyntaxError: Error #1132",exception21);

var exception22='no exception';
try {
    JSON.parse('fazse');
} catch (e) {
 exception22=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'fazse'","SyntaxError: Error #1132",exception22);

var exception23='no exception';
try {
    JSON.parse('falze');
} catch (e) {
 exception23=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'falze'","SyntaxError: Error #1132",exception23);

var exception24='no exception';
try {
    JSON.parse('falsz');
} catch (e) {
 exception24=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse close to null 'falsz'","SyntaxError: Error #1132",exception24);

/*
var exception25='no exception';
try {
    JSON.parse('[');
} catch (e) {
 exception25=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse invalid Array","SyntaxError: Error #1132",exception25);

var exception26='no exception';
try {
    JSON.parse('{');
} catch (e) {
 exception26=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse invalid Array","SyntaxError: Error #1132",exception26);
*/
var exception27='no exception';
try {
    JSON.parse('{"a"}');
} catch (e) {
 exception27=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse invalid Array","SyntaxError: Error #1132",exception27);

var exception28='no exception';
try {
    JSON.parse('{"a":}');
} catch (e) {
 exception28=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse invalid Array","SyntaxError: Error #1132",exception28);

var exception29='no exception';
try {
    JSON.parse('{"a":1,}');
} catch (e) {
 exception29=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse invalid Array","SyntaxError: Error #1132",exception29);

var exception30='no exception';
try {
    JSON.parse('fa');
} catch (e) {
 exception30=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse incomplete false","SyntaxError: Error #1132",exception30);

var exception31='no exception';
try {
    JSON.parse('tr');
} catch (e) {
 exception31=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse incomplete true","SyntaxError: Error #1132",exception31);

var exception32='no exception';
try {
    JSON.parse('nu');
} catch (e) {
 exception32=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse incomplete true","SyntaxError: Error #1132",exception32);

var exception33='no exception';
try {
    JSON.parse(null);
} catch (e) {
 exception33=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse null","SyntaxError: Error #1132",exception33);

var exception34='no exception';
try {
    JSON.parse(undefined);
} catch (e) {
 exception34=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse undefined","SyntaxError: Error #1132",exception34);

var exception35='no exception';
try {
    JSON.parse("[");
} catch (e) {
 exception35=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse undefined","SyntaxError: Error #1132",exception35);

var exception36='no exception';
try {
    JSON.parse("{");
} catch (e) {
 exception36=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse undefined","SyntaxError: Error #1132",exception36);


var exception37='no exception';
try {
    JSON.parse('[1,2,3]',1);
} catch (e) {
 exception37=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse, reviver is not a function","TypeError: Error #1034",exception37);

var reviverError="no exception";
try {
    JSON.parse('[1,2,3]',function f(key,value) {  throw new Error("Reviver Error"); });
} catch (e) {
    reviverError=e.toString();
}
Assert.expectEq("JSON parse, reviver throws error",reviverError,"Error: Reviver Error");

var cyclicError="no exception";
var cyclic=new Object();
cyclic.name="cyclic object";
cyclic.object=new Object();
cyclic.object.name="cyclic object";
cyclic.object.pointer=cyclic;

try {
   JSON.stringify(cyclic);
} catch (e) {
    cyclicError=removeExceptionDetail(e.toString());
}
Assert.expectEq("JSON parse, reviver throws error",cyclicError,"TypeError: Error #1129");

