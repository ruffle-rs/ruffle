/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "12.6.3-9-n";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The for..in statment";
    //var error = err;


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    //  for ( LeftHandSideExpression in Expression )
    //  LeftHandSideExpression:NewExpression:MemberExpression

    var o = new MyObject();
    var result = 0;

    var thisError = "no exception thrown";
    try{
        for ( var o in foo) {
            result += this[o];
        }
    } catch (e) {
        thisError = e.toString();
    } finally {
        array[item++] = Assert.expectEq( 
            "object is not defined",
            "ReferenceError: Error #1065",
            Utils.referenceError( thisError) );
    }
    return array;
}


function MyObject() {
    this.value = 2;
    this[0] = 4;
    return this;
}
