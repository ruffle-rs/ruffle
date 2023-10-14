/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
// var SECTION = "bug127913";

var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    function foo(){ foo2(); }
    function foo2(){ foo(); }
    
    try{
        unknownfunction();
    } catch(e) {
    } finally {
    }
    
    try{
        foo();
    } catch( e ){
    } finally {
        array[item++] = Assert.expectEq( "throw an exception in a script where there is infinite recursion", "no crash", "no crash" );
    }
    return array;
}

