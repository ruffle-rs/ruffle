/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.7.4.3";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array:Array = new Array();
    var item:Number= 0;
    var o:Number = new Number();
    
    array[item++] = Assert.expectEq( "Number.prototype.toLocaleString()",       "0",        Number.prototype.toLocaleString() );
    array[item++] = Assert.expectEq( "typeof(Number.prototype.toLocaleString())", "string",      typeof(Number.prototype.toLocaleString()) );
    var s:Number=new Number();

    var thisError:String = "no exception thrown";
    try{
        s = Number.prototype.toLocaleString;
         
        o.toLocaleString = s;
    } catch (e:ReferenceError) {
        thisError = e.toString();
    } finally {
        array[item++] = Assert.expectEq(  "s = Number.prototype.toLocaleString; o = new Number(); o.toLocaleString = s; o.toLocaleString()",
                                                "ReferenceError: Error #1056",
                                                Utils.referenceError( thisError ) );
    }

    thisError = "no exception thrown";
        var o:Number = new Number(1);
    try{
        s = Number.prototype.toLocaleString;
         
        o.toLocaleString = s;
    } catch(e1:ReferenceError) {
        thisError = e1.toString();
    } finally {
        array[item++] = Assert.expectEq(  "s = Number.prototype.toLocaleString; o = new Number(1); o.toLocaleString = s; o.toLocaleString()",
                                                "ReferenceError: Error #1056",
                                                Utils.referenceError( thisError) );
    }

    thisError = "no exception thrown";
        var o:Number= new Number(-1);
    try{
        s = Number.prototype.toLocaleString;
         
        o.toLocaleString = s;
    } catch (e2:ReferenceError) {
        thisError = e2.toString();
    } finally {
        array[item++] = Assert.expectEq(  "s = Number.prototype.toLocaleString; o = new Number(-1); o.toLocaleString = s; o.toLocaleString()",
                                                "ReferenceError: Error #1056",
                                                Utils.referenceError(thisError) );
    }

    var MYNUM = new Number(255);
    array[item++] = Assert.expectEq( "var MYNUM = new Number(255); MYNUM.toLocaleString()",          "255",      MYNUM.toLocaleString() );

    var MYNUM = new Number(Number.NaN);
    array[item++] = Assert.expectEq( "var MYNUM = new Number(Number.NaN); MYNUM.toLocaleString()",   "NaN",      MYNUM.toLocaleString() );

    var MYNUM = new Number(Infinity);
    array[item++] = Assert.expectEq( "var MYNUM = new Number(Infinity); MYNUM.toLocaleString()",   "Infinity",   MYNUM.toLocaleString() );

    var MYNUM = new Number(-Infinity);
    array[item++] = Assert.expectEq( "var MYNUM = new Number(-Infinity); MYNUM.toLocaleString()",   "-Infinity", MYNUM.toLocaleString() );

    var o=new Number(true);

    array[item++] = Assert.expectEq(   "o=new Number(true);o.toLocaleString()",                "1",o.toLocaleString() );
    
    o=new Number(false);
    array[item++] = Assert.expectEq(   "o=new Number(false);o.toLocaleString()",           "0",        o.toLocaleString() );
    o=new Number(new Array());
    array[item++] = Assert.expectEq(   "o=new Number(new Array());o.toLocaleString()",     "0",                o.toLocaleString() );
    o=new Number(Number.NaN);
    array[item++] = Assert.expectEq(     "o=Number.NaN;o.toLocaleString()",       "NaN",                  o.toLocaleString() );
    o=new Number(0)
    array[item++] = Assert.expectEq(     "o=0;o.toLocaleString()",                "0",                    o.toLocaleString());
    o=new Number(-0);
    array[item++] = Assert.expectEq(     "o=-0;o.toLocaleString()",               "0",                   o.toLocaleString() );

    o=new Number(Number.POSITIVE_INFINITY);
    array[item++] = Assert.expectEq(     "o=new Number(Number.POSITIVE_INFINITY)", "Infinity",     o.toLocaleString() );
    o=new Number(Number.NEGATIVE_INFINITY);
    array[item++] = Assert.expectEq(     "o=new Number(Number.NEGATIVE_INFINITY);o.toLocaleString()", "-Infinity",     o.toLocaleString() );
  
    o=new Number(-1);
    array[item++] = Assert.expectEq(     "o=new Number(-1);o.toLocaleString()", "-1",     o.toLocaleString() );

    // cases in step 6:  integers  1e21 > x >= 1 or -1 >= x > -1e21
   
    o=new Number(1);
    array[item++] = Assert.expectEq(     "o=new Number(1);o.toLocaleString()", "1",     o.toLocaleString() );
    o=new Number(10);
    array[item++] = Assert.expectEq(     "o=new Number(10);o.toLocaleString()", "10",     o.toLocaleString() );
    o=new Number(100);
    array[item++] = Assert.expectEq(     "o=new Number(100);o.toLocaleString()", "100",     o.toLocaleString() );
    o=new Number(1000);
    array[item++] = Assert.expectEq(     "o=new Number(1000);o.toLocaleString()", "1000",     o.toLocaleString() );
    o=new Number(10000);
    array[item++] = Assert.expectEq(     "o=new Number(10000);o.toLocaleString()", "10000",     o.toLocaleString() );
    o=new Number(10000000000);
    array[item++] = Assert.expectEq(     "o=new Number(10000000000);o.toLocaleString()", "10000000000",o.toLocaleString() );
    o=new Number(10000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(10000000000000000000);o.toString()", "10000000000000000000",o.toString() );
    o=new Number(100000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(100000000000000000000);o.toLocaleString()", "100000000000000000000",o.toLocaleString() );
    o=new Number(12345 );
    array[item++] = Assert.expectEq(     "o=new Number(12345 );o.toLocaleString()", "12345",o.toLocaleString() );
    o=new Number(1234567890 );
    array[item++] = Assert.expectEq(     "o=new Number(1234567890);o.toLocaleString()", "1234567890",o.toLocaleString() );
    o=new Number(-1);
    array[item++] = Assert.expectEq(     "o=new Number(-1);o.toLocaleString()", "-1",o.toLocaleString() );
    o=new Number(-10 );
    array[item++] = Assert.expectEq(     "o=new Number(-10 );o.toLocaleString()", "-10",o.toLocaleString() );
   
    o=new Number(-100 );
    array[item++] = Assert.expectEq(     "o=new Number(-100 );o.toLocaleString()", "-100",o.toLocaleString() );
    o=new Number(-1000 );
    array[item++] = Assert.expectEq(     "o=new Number(-1000 );o.toLocaleString()", "-1000",o.toLocaleString() );
    o=new Number(-1000000000 );
    array[item++] = Assert.expectEq(     "o=new Number(-1000000000 );o.toLocaleString()", "-1000000000",o.toLocaleString() );
    o=new Number(-1000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(-1000000000000000);o.toLocaleString()", "-1000000000000000",o.toLocaleString() );
    o=new Number(-100000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(-100000000000000000000);o.toLocaleString()", "-100000000000000000000",o.toLocaleString() );

    o=new Number(-1000000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(-1000000000000000000000);o.toLocaleString()", "-1e+21",o.toLocaleString() );

    o=new Number(1.0000001 );
    array[item++] = Assert.expectEq(     "o=new Number(1.0000001);o.toLocaleString()", "1.0000001",o.toLocaleString() );
    o=new Number(1000000000000000000000);
    array[item++] = Assert.expectEq(     "o=new Number(1000000000000000000000);o.toLocaleString()", "1e+21",o.toLocaleString() );

    o=new Number(1.2345);
    array[item++] = Assert.expectEq(     "o=new Number(1.2345);o.toLocaleString()", "1.2345",o.toLocaleString() );

    o=new Number(1.234567890);
    array[item++] = Assert.expectEq(     "o=new Number(1.234567890);o.toLocaleString()", "1.23456789",o.toLocaleString() );

    o=new Number(.12345);
    array[item++] = Assert.expectEq(     "o=new Number(.12345);o.toLocaleString()", "0.12345",o.toLocaleString() );

    o=new Number(.012345);
    array[item++] = Assert.expectEq(     "o=new Number(.012345);o.toLocaleString()", "0.012345",o.toLocaleString() );
    
    o=new Number(.0012345);
    array[item++] = Assert.expectEq(     "o=new Number(.0012345);o.toLocaleString()", "0.0012345",o.toLocaleString() );
    o=new Number(.00012345);
    array[item++] = Assert.expectEq(     "o=new Number(.00012345);o.toLocaleString()", "0.00012345",o.toLocaleString() );
    o=new Number(.000012345);
    array[item++] = Assert.expectEq(     "o=new Number(.000012345);o.toLocaleString()", "0.000012345",o.toLocaleString() );
    o=new Number(.0000012345);
    array[item++] = Assert.expectEq(     "o=new Number(.0000012345);o.toLocaleString()", "0.0000012345",o.toLocaleString() );
    o=new Number(.00000012345);
    array[item++] = Assert.expectEq(     "o=new Number(.00000012345);o.toLocaleString()", "1.2345e-7",o.toLocaleString() );
    o=new Number();
    array[item++] = Assert.expectEq(     "o=new Number();o.toLocaleString()", "0",o.toLocaleString() );
      

    return ( array );
}
