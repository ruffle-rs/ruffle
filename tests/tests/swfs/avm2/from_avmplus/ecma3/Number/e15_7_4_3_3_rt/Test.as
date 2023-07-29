/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.7.4.3-3-n";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();



function getTestCases() {
    var array = new Array();
    var item = 0;

//    array[item++] = Assert.expectEq("15.7.4.1", "v = Number.prototype.valueOf; num = 3; num.valueOf = v; num.valueOf()", "error",  "v = Number.prototype.valueOf; num = 3; num.valueOf = v; num.valueOf()" );
/*
    v = Number.prototype.valueOf;
    o = new String('Infinity');
    o.valueOf = v;
    array[item++] = Assert.expectEq("15.7.4.1", "v = Number.prototype.valueOf; o = new String('Infinity'); o.valueOf = v; o.valueOf()", "error",  o.valueOf() );*/
//    array[item++] = Assert.expectEq("15.7.4.1", "v = Number.prototype.valueOf; o = new Object(); o.valueOf = v; o.valueOf()", "error",  "v = Number.prototype.valueOf; o = new Object(); o.valueOf = v; o.valueOf()" );
    var v = Number.prototype.valueOf;
    var o = new String('Infinity');

    //TO-DO: commenting as3Enabled and changing expectedError to 1037
    var expectedError=1037;


   /* var expectedError = 1056;
   // if (as3Enabled) {
        expectedError = 1037;
    //}*/

    try{
        o.valueOf = v;
        o.valueOf();
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++] = Assert.expectEq("15.7.4.1 v = Number.prototype.valueOf; o = new String('Infinity'); o.valueOf = v; o.valueOf()",
                                    Utils.REFERENCEERROR+expectedError,
                                    Utils.referenceError(thisError) );
    }



    return ( array );
}
