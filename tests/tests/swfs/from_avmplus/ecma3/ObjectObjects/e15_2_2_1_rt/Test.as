/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.2.2.1";
//     var VERSION = "ECMA_4";
//     var TITLE   = "new Object( value )";


    var testcases = getTestCases();

   // TODO: REVIEW AS4 CONVERSION ISSUE 
 // Adding this function getJSClass directly to file rather than in Utils

 function getJSClass(obj)
{
  if (isObject(obj))
    return findClass(findType(obj));
  return cnNoObject;
}
function isObject(obj)
{
  return obj instanceof Object;
}

function findType(obj)
{
  var cnObjectToString = Object.prototype.toString;
  return cnObjectToString.apply(obj);
}
// given '[object Number]',  return 'Number'
function findClass(sType)
{
  var re =  /^\[.*\s+(\w+)\s*\]$/;
  var a = sType.match(re);

  if (a && a[1])
    return a[1];
  return cnNoClass;
}

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(   "typeof new Object(null)",      "object",           typeof new Object(null) );
    MYOB = new Object(null);
    MYOB.toString = Object.prototype.toString;
    array[item++] = Assert.expectEq(   "MYOB = new Object(null); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "[object Object]",    MYOB.toString());

    array[item++] = Assert.expectEq(   "typeof new Object(void 0)",      "object",           typeof new Object(void 0) );
    MYOB = new Object(new Object(void 0));
    MYOB.toString = Object.prototype.toString;
    array[item++] = Assert.expectEq(   "MYOB = new Object(new Object(void 0)); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "[object Object]",    MYOB.toString());

    array[item++] = Assert.expectEq(   "typeof new Object(undefined)",      "object",           typeof new Object(undefined) );
    MYOB = new Object(new Object(undefined));
    MYOB.toString = Object.prototype.toString;
    array[item++] = Assert.expectEq(   "MYOB = new Object(new Object(undefined)); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "[object Object]",    MYOB.toString());

    array[item++] = Assert.expectEq(   "typeof new Object('string')",      "string",           typeof new Object('string') );

    MYOB = new Object('string');
    array[item++] = Assert.expectEq(   "MYOB = (new Object('string'); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "string", MYOB.toString());

    array[item++] = Assert.expectEq(   "(new Object('string').valueOf()",  "string",           (new Object('string')).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object('')",            "string",           typeof new Object('') );
    MYOB = new Object('');
    array[item++] = Assert.expectEq(   "MYOB = (new Object(''); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "", MYOB.toString());

    array[item++] = Assert.expectEq(   "(new Object('').valueOf()",        "",                 (new Object('')).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object(Number.NaN)",      "number",                 typeof new Object(Number.NaN) );
    MYOB = new Object(Number.NaN);
    array[item++] = Assert.expectEq(   "MYOB = (new Object(Number.NaN); MYOB.toStriobjectng = Object.prototype.toString; MYOB.toString()",  "NaN", MYOB.toString() );
    array[item++] = Assert.expectEq(   "(new Object(Number.NaN).valueOf()",  Number.NaN,               (new Object(Number.NaN)).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object(0)",      "number",                 typeof new Object(0) );
    MYOB = new Object(0);
    array[item++] = Assert.expectEq(   "MYOB = (new Object(0); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "0", MYOB.toString());
    array[item++] = Assert.expectEq(   "(new Object(0).valueOf()",  0,               (new Object(0)).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object(-0)",      "number",                 typeof new Object(-0) );
    MYOB = new Object(-0);
    array[item++] = Assert.expectEq(   "MYOB = (new Object(-0); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "0", MYOB.toString() );
    array[item++] = Assert.expectEq(   "(new Object(-0).valueOf()",  -0,               (new Object(-0)).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object(1)",      "number",                 typeof new Object(1) );
    MYOB = new Object(1);
    array[item++] = Assert.expectEq(   "MYOB = (new Object(1); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "1",    MYOB.toString() );
    array[item++] = Assert.expectEq(   "(new Object(1).valueOf()",  1,               (new Object(1)).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object(-1)",      "number",                 typeof new Object(-1) );
    MYOB = new Object(-1);
    array[item++] = Assert.expectEq(   "MYOB = (new Object(-1); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "-1",    MYOB.toString() );
    array[item++] = Assert.expectEq(   "(new Object(-1).valueOf()",  -1,               (new Object(-1)).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object(true)",      "boolean",                 typeof new Object(true) );
    Boolean.prototype.valueOf=Object.prototype.valueOf;
    MYOB = new Object(true);
    array[item++] = Assert.expectEq(   "MYOB = (new Object(true); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "true",  MYOB.toString() );
    array[item++] = Assert.expectEq(   "(new Object(true).valueOf()",  true,               (new Object(true)).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object(false)",      "boolean",              typeof new Object(false) );
    MYOB = new Object(false);
    array[item++] = Assert.expectEq(   "MYOB = (new Object(false); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "false",  MYOB.toString() );
    array[item++] = Assert.expectEq(   "(new Object(false).valueOf()",  false,                 (new Object(false)).valueOf() );

    array[item++] = Assert.expectEq(   "typeof new Object(Boolean())",         "boolean",               typeof new Object(Boolean()) );
    MYOB = new Object(Boolean());
    array[item++] = Assert.expectEq(   "MYOB = (new Object(Boolean()); MYOB.toString = Object.prototype.toString; MYOB.toString()",  "false",  MYOB.toString() );
    array[item++] = Assert.expectEq(   "(new Object(Boolean()).valueOf()",     Boolean(),              (new Object(Boolean())).valueOf() );

    var myglobal    = this;
    var myobject    = new Object( "my new object" );
    var myarray     = new Array();
    var myboolean   = new Boolean();
    var mynumber    = new Number();
    var mystring    = new String();
    var myobject    = new Object();
    myfunction      = function(x){return x;}
    var mymath      = Math;
    var myregexp    = new RegExp(new String(''));

    function myobj():String{
        function f():String{
            return "hi!";
        }
    return f();
    }

    array[item++] = Assert.expectEq(  "myglobal = new Object( this )",                     myglobal,       new Object(this) );
    array[item++] = Assert.expectEq(  "myobject = new Object('my new object'); new Object(myobject)",            myobject,       new Object(myobject) );
    array[item++] = Assert.expectEq(  "myarray = new Array(); new Object(myarray)",        myarray,        new Object(myarray) );
    array[item++] = Assert.expectEq(  "myboolean = new Boolean(); new Object(myboolean)",  myboolean,      new Object(myboolean) );
    array[item++] = Assert.expectEq(  "mynumber = new Number(); new Object(mynumber)",     mynumber,       new Object(mynumber) );
    array[item++] = Assert.expectEq(  "mystring = new String(); new Object(mystring)",     mystring,       new Object(mystring) );
    array[item++] = Assert.expectEq(  "myobject = new Object(); new Object(myobject)",    myobject,       new Object(myobject) );
    array[item++] = Assert.expectEq(  "myfunction = function(x){return x;} new Object(myfunction)", myfunction,   new Object(myfunction) );
    array[item++] = Assert.expectEq(  "function myobj(){function f(){}return f() new Object(myobj)", myobj,   new Object(myobj) );
    array[item++] = Assert.expectEq(  "mymath = Math; new Object(mymath)",                 mymath,         new Object(mymath) );
    array[item++] = Assert.expectEq(  "myregexp = new RegExp(new String('')), new Object(myregexp)",                 myregexp,         new Object(myregexp) );

    return ( array );
}
