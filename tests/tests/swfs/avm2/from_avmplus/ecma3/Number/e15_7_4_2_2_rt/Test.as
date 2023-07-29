/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.7.4.2-2-n";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    o = new Object();
    o.toString = Number.prototype.toString;

    thisError="no error";
    try{
        o.toString();
    }catch(e:Error){
        thisError=(e.toString()).substring(0,18);
    }finally{trace(thisError);
        array[item++] = Assert.expectEq(  "o = new Object(); o.toString = Number.prototype.toString; o.toString()",  "TypeError: Error #",thisError );
    }
  /*array[item++] = Assert.expectEq(  "o = new Object(); o.toString = Number.prototype.toString; o.toString()",  "NaN",    o.toString(10) );*/
    //array[item++] = Assert.expectEq(  "o = new String(); o.toString = Number.prototype.toString; o.toString()",  "error",    "o = new String(); o.toString = Number.prototype.toString; o.toString()" );
    //array[item++] = Assert.expectEq(  "o = 3; o.toString = Number.prototype.toString; o.toString()",             "error",    "o = 3; o.toString = Number.prototype.toString; o.toString()" );

    return ( array );
}
