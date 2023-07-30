/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.5-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.charCodeAt";


    var TEST_STRING = new String( " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~" );

    var testcases = getTestCases();

function MyObject (v) {
    this.value      = v;
    //this.toString   = new Function ( "return this.value +\"\"" );
    this.toString   = function (){return this.value+'';}
    this.charCodeAt     = String.prototype.charCodeAt;
}

function getTestCases() {
    var array = new Array();
    var item = 0;

    var foo = new MyObject('hello');

    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello');foo.charCodeAt(0)", 0x0068, foo.charCodeAt(0)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello');foo.charCodeAt(1)", 0x0065, foo.charCodeAt(1)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello');foo.charCodeAt(2)", 0x006c, foo.charCodeAt(2)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello');foo.charCodeAt(3)", 0x006c, foo.charCodeAt(3)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello');foo.charCodeAt(4)", 0x006f, foo.charCodeAt(4)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello');foo.charCodeAt(-1)", Number.NaN,  foo.charCodeAt(-1)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello');foo.charCodeAt(5)", Number.NaN,  foo.charCodeAt(5)  );

    var boo = new MyObject(true);

    array[item++] = Assert.expectEq(  "var boo = new MyObject(true);boo.charCodeAt(0)", 0x0074, boo.charCodeAt(0)  );
    array[item++] = Assert.expectEq(  "var boo = new MyObject(true);boo.charCodeAt(1)", 0x0072, boo.charCodeAt(1)  );
    array[item++] = Assert.expectEq(  "var boo = new MyObject(true);boo.charCodeAt(2)", 0x0075, boo.charCodeAt(2)  );
    array[item++] = Assert.expectEq(  "var boo = new MyObject(true);boo.charCodeAt(3)", 0x0065, boo.charCodeAt(3)  );

    var noo = new MyObject( Math.PI );

    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI);noo.charCodeAt(0)", 0x0033, noo.charCodeAt(0)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI);noo.charCodeAt(1)", 0x002E, noo.charCodeAt(1)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI);noo.charCodeAt(2)", 0x0031, noo.charCodeAt(2)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI);noo.charCodeAt(3)", 0x0034, noo.charCodeAt(3)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI);noo.charCodeAt(4)", 0x0031, noo.charCodeAt(4)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI);noo.charCodeAt(5)", 0x0035, noo.charCodeAt(5)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI);noo.charCodeAt(6)", 0x0039, noo.charCodeAt(6)  );

    var noo = new MyObject( null );

    array[item++] = Assert.expectEq(  "var noo = new MyObject(null);noo.charCodeAt(0)", 0x006E, noo.charCodeAt(0)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(null);noo.charCodeAt(1)", 0x0075, noo.charCodeAt(1)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(null);noo.charCodeAt(2)", 0x006C, noo.charCodeAt(2)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(null);noo.charCodeAt(3)", 0x006C, noo.charCodeAt(3)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(null);noo.charCodeAt(4)", NaN, noo.charCodeAt(4)  );

    var noo = new MyObject( void 0 );

    array[item++] = Assert.expectEq(  "var noo = new MyObject(void 0);noo.charCodeAt(0)", 0x0075, noo.charCodeAt(0)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(void 0);noo.charCodeAt(1)", 0x006E, noo.charCodeAt(1)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(void 0);noo.charCodeAt(2)", 0x0064, noo.charCodeAt(2)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(void 0);noo.charCodeAt(3)", 0x0065, noo.charCodeAt(3)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(void 0);noo.charCodeAt(4)", 0x0066, noo.charCodeAt(4)  );

    return array;
}
