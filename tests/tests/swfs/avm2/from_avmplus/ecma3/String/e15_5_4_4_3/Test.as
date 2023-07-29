/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.5.4.4-3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.charAt";


    var testcases = getTestCases();

function MyObject (v) {
    this.value      = v;
    this.toString   = function() { return this.value +''; }
    this.valueOf    = function() { return this.value }
    this.charAt     = String.prototype.charAt;
}
function getTestCases() {
    var array = new Array();
    var item = 0;

    var foo = new MyObject('hello');


    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello'); ", "h", foo.charAt(0)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello'); ", "e", foo.charAt(1)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello'); ", "l", foo.charAt(2)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello'); ", "l", foo.charAt(3)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello'); ", "o", foo.charAt(4)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello'); ", "",  foo.charAt(-1)  );
    array[item++] = Assert.expectEq(  "var foo = new MyObject('hello'); ", "",  foo.charAt(5)  );

    var boo = new MyObject(true);

    array[item++] = Assert.expectEq(  "var boo = new MyObject(true); ", "t", boo.charAt(0)  );
    array[item++] = Assert.expectEq(  "var boo = new MyObject(true); ", "r", boo.charAt(1)  );
    array[item++] = Assert.expectEq(  "var boo = new MyObject(true); ", "u", boo.charAt(2)  );
    array[item++] = Assert.expectEq(  "var boo = new MyObject(true); ", "e", boo.charAt(3)  );

    var noo = new MyObject( Math.PI );

    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI); ", "3", noo.charAt(0)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI); ", ".", noo.charAt(1)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI); ", "1", noo.charAt(2)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI); ", "4", noo.charAt(3)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI); ", "1", noo.charAt(4)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI); ", "5", noo.charAt(5)  );
    array[item++] = Assert.expectEq(  "var noo = new MyObject(Math.PI); ", "9", noo.charAt(6)  );

    return array;
}
