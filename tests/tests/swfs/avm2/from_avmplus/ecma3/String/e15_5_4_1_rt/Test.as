/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
    //print( String( 10000000000000000000 ) )
//     var SECTION = "15.5.4.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.constructor";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "String.prototype.constructor == String",  true, String.prototype.constructor == String );
    thisError="no error";
    STRING = new String.prototype.constructor('hi');
    try{
        STRING.getClass = Object.prototype.toString;
        STRING.getClass();
    }catch(e:Error){
        thisError=e.toString();
    }finally{
        array[item++] = Assert.expectEq(  "var STRING = new String.prototype.constructor('hi'); STRING.getClass = Object.prototype.toString; STRING.getClass()","ReferenceError: Error #1056",Utils.referenceError(thisError));
    }

    /*array[item++] = Assert.expectEq(  "var STRING = new String.prototype.constructor('hi'); STRING.getClass = Object.prototype.toString; STRING.getClass()","[object String]",(STRING = new String.prototype.constructor('hi'), STRING.getClass = Object.prototype.toString, STRING.getClass() ) );*/
    return ( array );
}

