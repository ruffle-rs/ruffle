/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var VERSION = "ECMA_1";
//     var SECTION = "9.9-1";

    var tc= 0;
    var testcases = getTestCases();

//  all tests must call a function that returns an array of TestCase objects.

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "Object(true).valueOf()",    true,                   (Object(true)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(true)",       "boolean",               typeof Object(true) );
    array[item++] = Assert.expectEq(  "(Object(true)).__proto__",  Boolean.prototype,      (Object(true)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(false).valueOf()",    false,                  (Object(false)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(false)",      "boolean",               typeof Object(false) );
    array[item++] = Assert.expectEq(  "(Object(true)).__proto__",  Boolean.prototype,      (Object(true)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(0).valueOf()",       0,                      (Object(0)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(0)",          "number",               typeof Object(0) );
    array[item++] = Assert.expectEq(  "(Object(0)).__proto__",     Number.prototype,      (Object(0)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(-0).valueOf()",      -0,                     (Object(-0)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(-0)",         "number",               typeof Object(-0) );
    array[item++] = Assert.expectEq(  "(Object(-0)).__proto__",    Number.prototype,      (Object(-0)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(1).valueOf()",       1,                      (Object(1)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(1)",          "number",               typeof Object(1) );
    array[item++] = Assert.expectEq(  "(Object(1)).__proto__",     Number.prototype,      (Object(1)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(-1).valueOf()",      -1,                     (Object(-1)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(-1)",         "number",               typeof Object(-1) );
    array[item++] = Assert.expectEq(  "(Object(-1)).__proto__",    Number.prototype,      (Object(-1)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(Number.MAX_VALUE).valueOf()",    1.7976931348623157e308,         (Object(Number.MAX_VALUE)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.MAX_VALUE)",       "number",                       typeof Object(Number.MAX_VALUE) );
    array[item++] = Assert.expectEq(  "(Object(Number.MAX_VALUE)).__proto__",  Number.prototype,               (Object(Number.MAX_VALUE)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(Number.MIN_VALUE).valueOf()",     5e-324,           (Object(Number.MIN_VALUE)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.MIN_VALUE)",       "number",         typeof Object(Number.MIN_VALUE) );
    array[item++] = Assert.expectEq(  "(Object(Number.MIN_VALUE)).__proto__",  Number.prototype, (Object(Number.MIN_VALUE)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(Number.POSITIVE_INFINITY).valueOf()",    Number.POSITIVE_INFINITY,       (Object(Number.POSITIVE_INFINITY)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.POSITIVE_INFINITY)",       "number",                       typeof Object(Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(  "(Object(Number.POSITIVE_INFINITY)).__proto__",  Number.prototype,               (Object(Number.POSITIVE_INFINITY)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(Number.NEGATIVE_INFINITY).valueOf()",    Number.NEGATIVE_INFINITY,       (Object(Number.NEGATIVE_INFINITY)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.NEGATIVE_INFINITY)",       "number",            typeof Object(Number.NEGATIVE_INFINITY) );
    array[item++] = Assert.expectEq(  "(Object(Number.NEGATIVE_INFINITY)).__proto__",  Number.prototype,   (Object(Number.NEGATIVE_INFINITY)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object(Number.NaN).valueOf()",      Number.NaN,                (Object(Number.NaN)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.NaN)",         "number",                  typeof Object(Number.NaN) );
    array[item++] = Assert.expectEq(  "(Object(Number.NaN)).__proto__",    Number.prototype,          (Object(Number.NaN)).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object('a string').valueOf()",      "a string",         (Object("a string")).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object('a string')",         "string",           typeof (Object("a string")) );
    array[item++] = Assert.expectEq(  "(Object('a string')).__proto__",    String.prototype,   (Object("a string")).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object('').valueOf()",              "",                 (Object("")).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object('')",                 "string",           typeof (Object("")) );
    array[item++] = Assert.expectEq(  "(Object('')).__proto__",            String.prototype,   (Object("")).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object('\\r\\t\\b\\n\\v\\f').valueOf()",   "\r\t\b\n\v\f",   (Object("\r\t\b\n\v\f")).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object('\\r\\t\\b\\n\\v\\f')",      "string",           typeof (Object("\\r\\t\\b\\n\\v\\f")) );
    array[item++] = Assert.expectEq(  "(Object('\\r\\t\\b\\n\\v\\f')).__proto__", String.prototype,   (Object("\\r\\t\\b\\n\\v\\f")).constructor.prototype);

    array[item++] = Assert.expectEq(   "Object( '\\\'\\\"\\' ).valueOf()",      "\'\"\\",          (Object("\'\"\\")).valueOf() );
    array[item++] = Assert.expectEq(   "typeof Object( '\\\'\\\"\\' )",        "string",           typeof Object("\'\"\\") );
    array[item++] = Assert.expectEq(   "Object( '\\\'\\\"\\' ).__proto__",      String.prototype,   (Object("\'\"\\")).constructor.prototype);

    array[item++] = Assert.expectEq(  "Object( new MyObject(true) ).valueOf()",    true,           Object( new MyObject(true) ).valueOf());
    array[item++] = Assert.expectEq(  "typeof Object( new MyObject(true) )",       "object",       typeof Object( new MyObject(true))  );
    array[item++] = Assert.expectEq(  "(Object( new MyObject(true) )).toString()",  "[object Object]",       Object( new MyObject(true) ).toString());

    return ( array );
}

function MyObject( value ) {
    this.value = value;
   // this.valueOf = new Function ( "return this.value" );
    this.valueOf = function (){ return this.value; }
}
