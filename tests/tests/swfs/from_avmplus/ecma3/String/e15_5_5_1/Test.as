/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.5.5.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.length";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var thisError:String="no error";
    var s:String = new String();
    try{
        s.length=10;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]=Assert.expectEq("var s= new String();s.length=10","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }

    array[item++] = Assert.expectEq(   
                                    "var s = new String(); s.length",
                                    0,
                                    (s.length ) );

   

   

    var s = new String();
    var props = '';
    for ( var p in s )
    {
        props += p;
    }

    array[item++] = Assert.expectEq(   
                                    "var s = new String(); var props = ''; for ( var p in s ) {  props += p; };  props",
                                    '',
                                   s);
    

    var thisError = "no error"
    s = new String();
    try{
        delete s.length;
    }catch(e1:ReferenceError){
        thisError = e1.toString();
    }
    array[item++] = Assert.expectEq(   
                                    "var s = new String(); delete s.length",
                                    "ReferenceError: Error #1120",
                                    Utils.referenceError(thisError) );
    

    array[item++] = Assert.expectEq(   
                                    "var s = new String(); delete s.length",
                                    0,
                                    (s = new String(), s.length ) );
    s = new String('hello');
    try{
        delete s.length;
    }catch(e2:ReferenceError){
        thisError = e2.toString();
    }
    array[item++] = Assert.expectEq(   
                                    "var s = new String('hello'); delete s.length",
                                    "ReferenceError: Error #1120",
                                    Utils.referenceError(thisError) );
     
    array[item++] = Assert.expectEq(   
                                    "var s = new String('hello'); delete s.length; s.length",
                                    5,
                                    ( s.length ) );
    s='abcdefghijklmnopqrstuvwxyz';

    array[item++] = Assert.expectEq(   
                                    "delete s.length",
                                    "ReferenceError: Error #1120",
                                    Utils.referenceError(thisError) );
    try{
        s.length=10;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]=Assert.expectEq("var s= new String();s.length=10","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }

    array[item++] = Assert.expectEq(   
                                    "var s = new String('hello');s='abcdefghijklmnopqrstuvwxyz'; delete s.length; s.length",
                                    26,
                                    ( s.length ) );

 
  
    return array;

}
