/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "12.9.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The return statement";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var x = 3;
    var y = 4;
    function f() { var a = x; return a; }

    array[item++] = Assert.expectEq(    "SECTION var x = 3; function f() { var a = x;return a; };f()",3,f() );

    var x = 3;
    var y = 4;
    function g() { var a = x; return; }

    array[item++] = Assert.expectEq(    "SECTION var x = 3; function g() { var a = x;return; };g()",undefined,g() );

    var x = 3;
    var y = 4;
    function h() { var a = x; return a;a=x+y }

    array[item++] = Assert.expectEq(    "SECTION var x = 3; function h() { var a = x;return a; a=x+y};h()",3,h() );

    var x = 3;
    var y = 4;
    function I() { var a = x; return;a=x+y }

    array[item++] = Assert.expectEq(    "SECTION var x = 3; function I() { var a = x;return;a=x+y; };I()",undefined,I() );

    var x = 3;
    var y = 4;
    function J() { var a = x; return a=x+y; }

    array[item++] = Assert.expectEq(    "SECTION var x = 3; function f() { var a = x;return a=x+y; };J()",7,J() );

    return array;

}
