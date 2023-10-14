/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION = "function";       // provide a document reference (ie, ECMA section)
// var VERSION = "";  // Version of JavaScript or ECMA
// var TITLE   = "test";       // Provide ECMA section title or a description

var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var f:Function = function():void{}
    f.someProperty = new Array(1,2,3);

    array[item++] = Assert.expectEq( "var f:Function = function():void{}; f.someProperty=new Array(1,2,3,)", [1,2,3]+"", f.someProperty+"" );
    
     //test case for bug 168157

     var f = function factorial (x) {
      if (x < 1) return 1; else return x * factorial(x-1);
     }

    array[item++] = Assert.expectEq( "The optional identifier for a function expression should be valid within the body of the function", 40320, f(8) );
    

    return array;
}
