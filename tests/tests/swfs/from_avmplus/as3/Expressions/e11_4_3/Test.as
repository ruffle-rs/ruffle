/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}
import com.adobe.test.Assert;

//     var SECTION = "e11_4_3";

//     var VERSION = "ECMA_1";

//     var TITLE   = " The typeof operator";

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(      "typeof(void(0))",              "undefined",        typeof(void(0)) );
    array[item++] = Assert.expectEq(      "typeof(null)",                 "object",           typeof(null) );
    array[item++] = Assert.expectEq(      "typeof(true)",                 "boolean",          typeof(true) );
    array[item++] = Assert.expectEq(      "typeof(false)",                "boolean",          typeof(false) );
    array[item++] = Assert.expectEq(      "typeof(new Boolean())",        "boolean",           typeof(new Boolean()) );
    array[item++] = Assert.expectEq(      "typeof(new Boolean(true))",    "boolean",           typeof(new Boolean(true)) );
    array[item++] = Assert.expectEq(      "typeof(Boolean())",            "boolean",          typeof(Boolean()) );
    array[item++] = Assert.expectEq(      "typeof(Boolean(false))",       "boolean",          typeof(Boolean(false)) );
    array[item++] = Assert.expectEq(      "typeof(Boolean(true))",        "boolean",          typeof(Boolean(true)) );
    array[item++] = Assert.expectEq(      "typeof(NaN)",                  "number",           typeof(Number.NaN) );
    array[item++] = Assert.expectEq(      "typeof(Infinity)",             "number",           typeof(Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(      "typeof(-Infinity)",            "number",           typeof(Number.NEGATIVE_INFINITY) );
    array[item++] = Assert.expectEq(      "typeof(Math.PI)",              "number",           typeof(Math.PI) );
    array[item++] = Assert.expectEq(      "typeof(0)",                    "number",           typeof(0) );
    array[item++] = Assert.expectEq(      "typeof(1)",                    "number",           typeof(1) );
    array[item++] = Assert.expectEq(      "typeof(-1)",                   "number",           typeof(-1) );
    array[item++] = Assert.expectEq(      "typeof('0')",                  "string",           typeof("0") );
    array[item++] = Assert.expectEq(      "typeof(Number())",             "number",           typeof(Number()) );
    array[item++] = Assert.expectEq(      "typeof(Number(0))",            "number",           typeof(Number(0)) );
    array[item++] = Assert.expectEq(      "typeof(Number(1))",            "number",           typeof(Number(1)) );
    array[item++] = Assert.expectEq(      "typeof(Nubmer(-1))",           "number",           typeof(Number(-1)) );
    array[item++] = Assert.expectEq(      "typeof(new Number())",         "number",           typeof(new Number()) );
    array[item++] = Assert.expectEq(      "typeof(new Number(0))",        "number",           typeof(new Number(0)) );
    array[item++] = Assert.expectEq(      "typeof(new Number(1))",        "number",           typeof(new Number(1)) );

    // Math does not implement [[Construct]] or [[Call]] so its type is object.

    array[item++] = Assert.expectEq(      "typeof(Math)",                 "object",         typeof(Math) );

    array[item++] = Assert.expectEq(      "typeof(Number.prototype.toString)", "function",    typeof(Number.prototype.toString) );

    array[item++] = Assert.expectEq(      "typeof('a string')",           "string",           typeof("a string") );
    array[item++] = Assert.expectEq(      "typeof('')",                   "string",           typeof("") );
    array[item++] = Assert.expectEq(      "typeof(new Date())",           "object",           typeof(new Date()) );
    array[item++] = Assert.expectEq(      "typeof(new Array(1,2,3))",     "object",           typeof(new Array(1,2,3)) );
    array[item++] = Assert.expectEq(      "typeof(new String('string object'))",  "string",   typeof(new String("string object")) );
    array[item++] = Assert.expectEq(      "typeof(String('string primitive'))",    "string",  typeof(String("string primitive")) );
    array[item++] = Assert.expectEq(      "typeof(['array', 'of', 'strings'])",   "object",   typeof(["array", "of", "strings"]) );
    array[item++] = Assert.expectEq(      "typeof(new Function())",                "function",     typeof( new Function() ) );
    array[item++] = Assert.expectEq(      "typeof(parseInt)",                      "function",     typeof( parseInt ) );
    array[item++] = Assert.expectEq(      "typeof(test)",                          "function",     typeof( Assert.expectEq ) );
    array[item++] = Assert.expectEq(      "typeof(String.fromCharCode)",           "function",     typeof( String.fromCharCode )  );

    var notype;
    array[item++] = Assert.expectEq(      "typeof var notype; ",              "undefined",        typeof notype );
    var hnumber = 0x464d34;
    array[item++] = Assert.expectEq(      "typeof hnumber = 0x464d34; ",              "number",        typeof hnumber );
    var obj:Object;
    array[item++] = Assert.expectEq(      "typeof var obj:Object; ",              "object",        typeof obj );
    obj = new Object();
    array[item++] = Assert.expectEq(      "typeof obj = new Object()",              "object",        typeof obj );
    var dt:Date;
    array[item++] = Assert.expectEq(      "typeof var dt:Date; ",              "object",        typeof dt );
    dt = new Date();
    array[item++] = Assert.expectEq(      "typeof dt = new Date()",              "object",        typeof dt );
    var localClass:LocalClass;
    array[item++] = Assert.expectEq(      "typeof localClass:LocalClass; ",              "object",        typeof localClass );
    localClass = new LocalClass();
    array[item++] = Assert.expectEq(      "typeof localClass = new LocalClass(); ",              "object",        typeof localClass );
    array[item++] = Assert.expectEq(      "typeof undefined",              "undefined",        typeof undefined );
    array[item++] = Assert.expectEq(      "typeof void(0)",              "undefined",        typeof void(0) );
    array[item++] = Assert.expectEq(      "typeof null",                 "object",           typeof null );
    array[item++] = Assert.expectEq(      "typeof true",                 "boolean",          typeof true );
    array[item++] = Assert.expectEq(      "typeof false",                "boolean",          typeof false );
    array[item++] = Assert.expectEq(      "typeof new Boolean()",        "boolean",           typeof new Boolean() );
    array[item++] = Assert.expectEq(      "typeof new Boolean(true)",    "boolean",           typeof new Boolean(true) );
    array[item++] = Assert.expectEq(      "typeof Boolean()",            "boolean",          typeof Boolean() );
    array[item++] = Assert.expectEq(      "typeof Boolean(false)",       "boolean",          typeof Boolean(false) );
    array[item++] = Assert.expectEq(      "typeof Boolean(true)",        "boolean",          typeof Boolean(true) );
    array[item++] = Assert.expectEq(      "typeof Number.NaN",                             "number",           typeof Number.NaN );
    array[item++] = Assert.expectEq(      "typeof Number.POSITIVE_INFINITY",             "number",           typeof Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(      "typeof Number.NEGATIVE_INFINITY",               "number",           typeof Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(      "typeof Math.PI",              "number",           typeof Math.PI );
    array[item++] = Assert.expectEq(      "typeof 0",                    "number",           typeof 0 );
    array[item++] = Assert.expectEq(      "typeof 1",                    "number",           typeof 1 );
    array[item++] = Assert.expectEq(      "typeof -1",                   "number",           typeof -1 );
    array[item++] = Assert.expectEq(      "typeof ''",                  "string",           typeof '' );
    array[item++] = Assert.expectEq(      "typeof '0'",                  "string",           typeof '0' );
    array[item++] = Assert.expectEq(      "typeof Number()",             "number",           typeof Number() );
    array[item++] = Assert.expectEq(      "typeof Number(0)",            "number",           typeof Number(0) );
    array[item++] = Assert.expectEq(      "typeof Number(1)",            "number",           typeof Number(1) );
    array[item++] = Assert.expectEq(      "typeof Nubmer(-1)",           "number",           typeof Number(-1) );
    array[item++] = Assert.expectEq(      "typeof new Number()",         "number",           typeof new Number() );
    array[item++] = Assert.expectEq(      "typeof new Number(0)",        "number",           typeof new Number(0) );
    array[item++] = Assert.expectEq(      "typeof new Number(1)",        "number",           typeof new Number(1) );
    array[item++] = Assert.expectEq(      "typeof Math",                 "object",           typeof Math );
    array[item++] = Assert.expectEq(      "typeof Number.prototype.toString", "function",    typeof Number.prototype.toString );
    array[item++] = Assert.expectEq(      "typeof String(\"a string\")",           "string",           typeof String("a string") );
    array[item++] = Assert.expectEq(      "typeof String(\"\")",                   "string",           typeof String("") );
    array[item++] = Assert.expectEq(      "typeof String(\"  \")",                   "string",           typeof String("  ") );
    array[item++] = Assert.expectEq(      "typeof new Date()",           "object",           typeof new Date() );
    array[item++] = Assert.expectEq(      "typeof new Array(1,2,3)",     "object",           typeof new Array(1,2,3) );
    array[item++] = Assert.expectEq(      "typeof new String('string object')",  "string",   typeof new String("string object") );
    array[item++] = Assert.expectEq(      "typeof String('string primitive')",    "string",  typeof String("string primitive") );
    array[item++] = Assert.expectEq(      "typeof ['array', 'of', 'strings']",   "object",   typeof ["array", "of", "strings"] );
    array[item++] = Assert.expectEq(      "typeof new Function()",                "function",     typeof new Function() );
    array[item++] = Assert.expectEq(      "typeof parseInt",                      "function",     typeof parseInt  );
    array[item++] = Assert.expectEq(      "typeof test",                          "function",     typeof Assert.expectEq );
    array[item++] = Assert.expectEq(      "typeof String.fromCharCode",           "function",     typeof String.fromCharCode  );

    return array;
}

class LocalClass {
    function LocalClass() {
        trace("Constructor of LocalClass");
    }
}