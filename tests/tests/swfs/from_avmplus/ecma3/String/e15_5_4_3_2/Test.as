/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "15.5.4.3-2";
//     var VERSION = "ECMA_4";
//     var TITLE   = "String.prototype.valueOf";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    valof=String.prototype.valueOf;
    String.prototype.valueOf=valof;

    var astring=new String();
    array[item++] = Assert.expectEq( 
                                  "valof=String.prototype.valueOf,String.prototype.valueOf=valof,astring=new String(), astring.valueOf()",
                                  "",
                                  (astring=new String(), astring.valueOf()) );
    array[item++] = Assert.expectEq( 
                                  "valof=String.prototype.valueOf,String.prototype.valueOf=valof,astring=new String(0), astring.valueOf()",
                                  "0",
                                  (astring=new String(0), astring.valueOf()) );
    array[item++] = Assert.expectEq( 
                                  "valof=String.prototype.valueOf,String.prototype.valueOf=valof,astring=new String('hello'), astring.valueOf()",
                                  "hello",
                                  (astring=new String('hello'), astring.valueOf()) );
    array[item++] = Assert.expectEq( 
                                  "valof=String.prototype.valueOf,String.prototype.valueOf=valof,astring=new String(''), astring.valueOf()",
                                  "",
                                  (astring=new String(''), astring.valueOf()) );

    return ( array );
}
