/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.5.4.9";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.localeCompare";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    
    var str1 = new String("h");
    var str2 = new String ("H");
    array[item++] = Assert.expectEq(   "str1.localeCompare(str2)", 32,  str1.localeCompare(str2) );

    var str2 = new String ("hello");
    array[item++] = Assert.expectEq(   "str1.localeCompare(str2)", -1,  str1.localeCompare(str2) );

    var str2 = new String ("flash");
    array[item++] = Assert.expectEq(   "str1.localeCompare(str2)", 2,  str1.localeCompare(str2) )
    
    var str1 = "h"
    var str2 = "H"
    var thisError = "no error";
    try{
       String.prototype.localeCompare(str2,str1)
    }catch(e:ArgumentError){
       thisError = (e.toString()).substring(0,26);
    }finally{
         array[item++] = Assert.expectEq(   "String.prototype.localeCompare(str2,str1)","ArgumentError: Error #1063",thisError)
    }

    /*var str2 = "hello";
    array[item++] = Assert.expectEq(   "localeCompare(str1,str2)",-13,  String.prototype.localeCompare(str1,str2) )

    var str2 = "flash"
    array[item++] = Assert.expectEq(   "localeCompare(str1,str2)",-11,  String.prototype.localeCompare(str2,str1) )*/
    

    return ( array );
}
