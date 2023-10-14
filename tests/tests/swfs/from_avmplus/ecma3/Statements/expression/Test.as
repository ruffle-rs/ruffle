/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = '12.4';
//     var VERSION = 'no version';
//     var TITLE = 'Statement:expression';



    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
            
    var t;
    var x = 3;
    var y= 5;
    ExpressionStatement: t = x + y;
            
    
    array[item++] = Assert.expectEq(  "Expression:{t}", 8, t);
    return array;
}
