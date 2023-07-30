/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.7.4.2-4";
//     var VERSION = "ECMA_4";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    
    var o = 3;
    array[item++] = Assert.expectEq("o = 3;o.toString()",
                        "3",
                        o.toString() );

    var o = new Number(3);
    array[item++] = Assert.expectEq( "o = new Number(3);o.toString()",
                        "3",
                        o.toString() );

    var o = new Number();
    array[item++] = Assert.expectEq("o = new Number();o.toString()",
                        "0",
                        o.toString() );

   
    var o = new Number(3);
    array[item++] = Assert.expectEq("o = new Number(3);o.toString()",
                        "3",
                        o.toString(10) );

    o = new Number(3);
    array[item++] = Assert.expectEq("o = new Number(3);o.toString()",
                        "11",
                        o.toString(2) );

    o = new Number(3);
    array[item++] = Assert.expectEq("o = new Number(3);o.toString()",
                        "3",
                        o.toString(8) );

    o = new Number(11);
    array[item++] = Assert.expectEq("o = new Number(11);o.toString()",
                        "13",
                        o.toString(8) );
   
    o= new Number(3);
    array[item++] = Assert.expectEq("o = new Number(3);o.toString()",
                        "3",
                        o.toString(16) );

    o= new Number(17);
    array[item++] = Assert.expectEq("o = new Number(17);o.toString()",
                        "11",
                        o.toString(16) );

    o=new Number(true);

    array[item++] = Assert.expectEq(   "o=new Number(true)",               "1",o.toString() );
    
    o=new Number(false);
    array[item++] = Assert.expectEq(   "o=new Number(false)",          "0",        o.toString() );
    o=new Number(new Array());
    array[item++] = Assert.expectEq(   "o=new Number(new Array())",        "0",                o.toString() );
    o=Number.NaN;
    array[item++] = Assert.expectEq(     "o=Number.NaN;o.toString()",       "NaN",                  o.toString() );
    o=new Number(0)
    array[item++] = Assert.expectEq(     "o=0;o.toString()",                "0",                    o.toString());
    o=new Number(-0);
    array[item++] = Assert.expectEq(     "o=-0;o.toString()",               "0",                   o.toString() );

    o=new Number(Number.POSITIVE_INFINITY);
    array[item++] = Assert.expectEq(     "o=new Number(Number.POSITIVE_INFINITY)", "Infinity",     o.toString() );
    o=new Number(Number.NEGATIVE_INFINITY);
    array[item++] = Assert.expectEq(     "o=new Number(Number.NEGATIVE_INFINITY);o.toString()", "-Infinity",     o.toString() );
  
    o=new Number(-1);
    array[item++] = Assert.expectEq(     "o=new Number(-1);o.toString()", "-1",     o.toString() );

    // cases in step 6:  integers  1e21 > x >= 1 or -1 >= x > -1e21
   
    o=new Number(1);
    array[item++] = Assert.expectEq(     "o=new Number(1);o.toString()", "1",     o.toString() );
    o=new Number(10);
    array[item++] = Assert.expectEq(     "o=new Number(10);o.toString()", "10",     o.toString() );
    o=new Number(100);
    array[item++] = Assert.expectEq(     "o=new Number(100);o.toString()", "100",     o.toString() );
    o=new Number(1000);
    array[item++] = Assert.expectEq(     "o=new Number(1000);o.toString()", "1000",     o.toString() );
    o=new Number(10000);
    array[item++] = Assert.expectEq(     "o=new Number(10000);o.toString()", "10000",     o.toString() );
    o=new Number(10000000000);
    array[item++] = Assert.expectEq(     "o=new Number(10000000000);o.toString()", "10000000000",o.toString() );
    o=new Number(10000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(10000000000000000000);o.toString()", "10000000000000000000",o.toString() );
    o=new Number(100000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(100000000000000000000);o.toString()", "100000000000000000000",o.toString() );
    o=new Number(12345 );
    array[item++] = Assert.expectEq(     "o=new Number(12345 );o.toString()", "12345",o.toString() );
    o=new Number(1234567890 );
    array[item++] = Assert.expectEq(     "o=new Number(1234567890);o.toString()", "1234567890",o.toString() );
    o=new Number(-1);
    array[item++] = Assert.expectEq(     "o=new Number(-1);o.toString()", "-1",o.toString() );
    o=new Number(-10 );
    array[item++] = Assert.expectEq(     "o=new Number(-10 );o.toString()", "-10",o.toString() );
   
    o=new Number(-100 );
    array[item++] = Assert.expectEq(     "o=new Number(-100 );o.toString()", "-100",o.toString() );
    o=new Number(-1000 );
    array[item++] = Assert.expectEq(     "o=new Number(-1000 );o.toString()", "-1000",o.toString() );
    o=new Number(-1000000000 );
    array[item++] = Assert.expectEq(     "o=new Number(-1000000000 );o.toString()", "-1000000000",o.toString() );
    o=new Number(-1000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(-1000000000000000);o.toString()", "-1000000000000000",o.toString() );
    o=new Number(-100000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(-100000000000000000000);o.toString()", "-100000000000000000000",o.toString() );
    
    o=new Number(-1000000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(-1000000000000000000000);o.toString()", "-1e+21",o.toString() );
    
    o=new Number(1.0000001 );
    array[item++] = Assert.expectEq(     "o=new Number(1.0000001);o.toString()", "1.0000001",o.toString() );
    o=new Number(1000000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(1000000000000000000000);o.toString()", "1e+21",o.toString() );

    o=new Number(1.2345);
    array[item++] = Assert.expectEq(     "o=new Number(1.2345);o.toString()", "1.2345",o.toString() );

    o=new Number(1.234567890);
    array[item++] = Assert.expectEq(     "o=new Number(1.234567890);o.toString()", "1.23456789",o.toString() );

    o=new Number(.12345);
    array[item++] = Assert.expectEq(     "o=new Number(.12345);o.toString()", "0.12345",o.toString() );

    o=new Number(.012345);
    array[item++] = Assert.expectEq(     "o=new Number(.012345);o.toString()", "0.012345",o.toString() );
    
    o=new Number(.0012345);
    array[item++] = Assert.expectEq(     "o=new Number(.0012345);o.toString()", "0.0012345",o.toString() );
    o=new Number(.00012345);
    array[item++] = Assert.expectEq(     "o=new Number(.00012345);o.toString()", "0.00012345",o.toString() );
    o=new Number(.000012345);
    array[item++] = Assert.expectEq(     "o=new Number(.000012345);o.toString()", "0.000012345",o.toString() );
    o=new Number(.0000012345);
    array[item++] = Assert.expectEq(     "o=new Number(.0000012345);o.toString()", "0.0000012345",o.toString() );
    o=new Number(.00000012345);
    array[item++] = Assert.expectEq(     "o=new Number(.00000012345);o.toString()", "1.2345e-7",o.toString() );
    o=new Number();
    array[item++] = Assert.expectEq(     "o=new Number();o.toString()", "0",o.toString() );
      
    var date = new Date(0);
    var thisError="no error";
    try{
        date.myToString=Number.prototype.toString
        date.myToString();
    }catch(e:TypeError){
        thisError = e.toString()
    }
   
    array[item++] = Assert.expectEq(
                        "date.myToString=Number.prototype.toString;date.myToString()",
                        "TypeError: Error #1004",
                        Utils.typeError(thisError) );

    var o = new Number(3);

    try{
        o.toString(1);
    }catch(e:RangeError){
        thisError=e.toString();
    }

    array[item++] = Assert.expectEq(
                        "var o=new Number(3);o.toString(1)","RangeError: Error #1003",
                        Utils.rangeError(thisError) );

    var o = new Number(3);

    try{
        o.toString(37);
    }catch(e:RangeError){
        thisError=e.toString();
    }

    array[item++] = Assert.expectEq(
                        "var o=new Number(3);o.toString(37)","RangeError: Error #1003",
                        Utils.rangeError(thisError) );

   

    return ( array );
}
