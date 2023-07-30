/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = '12.1';
//     var VERSION = 'no version';
//     var TITLE = 'Statement:block';


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var t;
    Block:
    {
            var x = 3;
            var y= 5;
            t = x + y;
    
    }
    
    array[item++] = Assert.expectEq(  "Block:{t}", 8, t);

    {
        var k = 100;
        var l = 50;
        thisError ="no error";
        try{
           t=k/l;
           t= n+10;
        }catch(e:ReferenceError){
            thisError=e.toString();
        }
        k=k+50;
            

    }

    array[item++] = Assert.expectEq(  "Block in which exception is thrown", "ReferenceError: Error #1065", Utils.referenceError(thisError));

    array[item++] = Assert.expectEq(  "Block in which exception is thrown",2, t);
    array[item++] = Assert.expectEq(  "Block in which exception is thrown",150, k);
       
    return array;
}
